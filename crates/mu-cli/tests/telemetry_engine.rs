//! The telemetry of the engine under the layers of `mu server`, with in-memory exporters.

use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use futures::StreamExt as _;
use mu_cli::telemetry::Providers;
use mu_core::{
    Backoff, Command, Event, EventKind, Journal, Keepalive, KeepaliveMode, MemoryJournal, NewEvent,
    Portal, Reconnect, ResolvedWorld, Seq, Source, WorldSetup, WorldState,
};
use mu_fake_server::{Script, ScriptedGame};
use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::InMemoryMetricExporter;
use opentelemetry_sdk::metrics::data::{AggregatedMetrics, MetricData};
use opentelemetry_sdk::trace::{InMemorySpanExporter, SpanData};
use tokio::runtime::Builder;
use tokio::sync::watch;
use tracing::{Dispatch, Subscriber};
use tracing_subscriber::layer::SubscriberExt as _;

use engine::{
    BrokenTransport, bare_world, broken_world, id, idle, on, portal, started, wait_event,
    wait_state, world,
};

#[path = "../../mu-core/tests/support/mod.rs"]
mod engine;

const PW: &str = "s3cret-pw";

/// The lock that runs the tests of this file one at a time. tracing builds the interest of each
/// callsite for the whole process from each subscriber that is alive, so the subscriber of one
/// test can change what `enabled!` returns in another. A span holds its subscriber, so the
/// subscriber of a test stays alive until its runtime drops the tasks that hold its spans.
static SERIAL: Mutex<()> = Mutex::new(());

/// Runs `body` under [`SERIAL`], with `subscriber` as the default of this thread, on a runtime of
/// its own.
fn run<T>(subscriber: impl Subscriber + Send + Sync + 'static, body: impl Future<Output = T>) -> T {
    run_on(
        Builder::new_current_thread(),
        Some(Box::new(subscriber)),
        body,
    )
}

/// [`run`] on a runtime whose clock is paused.
fn run_paused<T>(
    subscriber: impl Subscriber + Send + Sync + 'static,
    body: impl Future<Output = T>,
) -> T {
    let mut builder = Builder::new_current_thread();
    builder.start_paused(true);
    run_on(builder, Some(Box::new(subscriber)), body)
}

/// [`run`] with no default subscriber.
fn run_without<T>(body: impl Future<Output = T>) -> T {
    run_on(Builder::new_current_thread(), None, body)
}

/// Makes the subscriber and the runtime, and ends them, while this thread holds [`SERIAL`]: the
/// runtime ends first, then the subscriber, then the lock.
fn run_on<T>(
    mut builder: Builder,
    subscriber: Option<Box<dyn Subscriber + Send + Sync>>,
    body: impl Future<Output = T>,
) -> T {
    let _lock = SERIAL.lock().unwrap_or_else(PoisonError::into_inner);
    let dispatch = subscriber.map(Dispatch::new);
    let _default = dispatch.as_ref().map(tracing::dispatcher::set_default);
    let runtime = builder.enable_all().build().unwrap();
    runtime.block_on(body)
}

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
        self.providers.force_flush().unwrap();
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
        self.providers.force_flush().unwrap();
        self.spans.get_finished_spans().unwrap()
    }
}

#[test]
fn other_targets_make_no_span_event() {
    let memory = Memory::new();
    run(memory.subscriber(), async {
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
    run(memory.subscriber(), async {
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
    run(memory.subscriber(), async {
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
    run(memory.subscriber(), async {
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
        streams: mu_config::stream::Streams::main(),
    })
}

/// A journal of `capacity` events, the sender of the stop, and an event stream from the first
/// event, past its `Hello`.
async fn journal_stream(
    capacity: usize,
) -> (
    Arc<MemoryJournal>,
    watch::Sender<bool>,
    connectrpc::ServiceStream<mu_proto::SubscribeResponse>,
) {
    let journal = Arc::new(MemoryJournal::new(capacity).unwrap());
    let (stop, stream) = stream_of(journal.clone()).await;
    (journal, stop, stream)
}

/// The sender of the stop, and an event stream of `journal` from the first event, past its
/// `Hello`.
async fn stream_of(
    journal: Arc<dyn Journal>,
) -> (
    watch::Sender<bool>,
    connectrpc::ServiceStream<mu_proto::SubscribeResponse>,
) {
    let (stop, stopping) = watch::channel(false);
    let subscription = journal.subscribe(Seq(1)).unwrap();
    let mut stream = mu_server::service::event_stream(subscription, journal, "id", stopping);
    stream.next().await.unwrap().unwrap();
    (stop, stream)
}

/// A journal that counts the calls of `last`.
struct Counted {
    journal: MemoryJournal,
    lasts: std::sync::atomic::AtomicUsize,
}

impl Journal for Counted {
    fn append(&self, event: NewEvent) -> Event {
        self.journal.append(event)
    }

    fn subscribe(&self, from: Seq) -> Result<mu_core::Subscription, mu_core::PortalError> {
        self.journal.subscribe(from)
    }

    fn last(&self) -> Seq {
        self.lasts.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.journal.last()
    }
}

/// The number of reads of the journal head while a stream sends one event.
async fn lasts_for_one_event() -> usize {
    let journal = Arc::new(Counted {
        journal: MemoryJournal::new(4).unwrap(),
        lasts: std::sync::atomic::AtomicUsize::new(0),
    });
    let (_stop, mut stream) = stream_of(journal.clone()).await;
    line(&journal.journal, "a line");
    stream.next().await.unwrap().unwrap();
    journal.lasts.load(std::sync::atomic::Ordering::SeqCst)
}

#[test]
fn lag_is_read_once_per_event_with_the_export() {
    let memory = Memory::new();
    run(memory.subscriber(), async {
        assert_eq!(lasts_for_one_event().await, 1);
    });
}

#[test]
fn lag_is_not_read_with_only_the_traces() {
    let providers = traces_only();
    run(providers.subscriber(), async {
        assert_eq!(lasts_for_one_event().await, 0);
    });
}

#[test]
fn lag_is_not_read_with_the_export_off() {
    let providers = Providers::none();
    run(providers.subscriber(), async {
        assert_eq!(lasts_for_one_event().await, 0);
    });
}

/// The providers of an export of the traces alone, to memory.
fn traces_only() -> Providers {
    Providers::with_exporters(
        Some(InMemorySpanExporter::default()),
        None::<InMemoryMetricExporter>,
        "mu",
    )
}

/// The stderr lines keep `WARN` of every target, also of a target that no export layer takes.
#[test]
fn warn_lines_stay_with_any_set_of_layers() {
    let cases = [
        ("the export off", Providers::none()),
        ("only the traces", traces_only()),
        (
            "only the metrics",
            Providers::with_exporters(
                None::<InMemorySpanExporter>,
                Some(InMemoryMetricExporter::default()),
                "mu",
            ),
        ),
    ];
    for (case, providers) in cases {
        run(providers.subscriber(), async {
            assert!(
                tracing::enabled!(target: "mu_core", tracing::Level::WARN),
                "{case}"
            );
            assert!(
                tracing::enabled!(target: "opentelemetry_sdk", tracing::Level::WARN),
                "{case}"
            );
        });
    }
}

#[test]
fn lag_is_not_read_without_a_subscriber() {
    run_without(async {
        assert_eq!(lasts_for_one_event().await, 0);
    });
}

#[test]
fn lagged_stream_is_counted() {
    let memory = Memory::new();
    run(memory.subscriber(), async {
        let (journal, _stop, mut stream) = journal_stream(4).await;
        for n in 1..=5 {
            line(&journal, &format!("line {n}"));
        }
        let error = stream.next().await.unwrap().unwrap_err();
        assert_eq!(error.code, connectrpc::ErrorCode::ResourceExhausted);
        assert_eq!(memory.value("mu.subscriber.lagged", &[]), Some(1));
    });
}

#[test]
fn lag_at_the_head_is_zero() {
    let memory = Memory::new();
    run(memory.subscriber(), async {
        let (journal, _stop, mut stream) = journal_stream(4).await;
        line(&journal, "the head");
        stream.next().await.unwrap().unwrap();
        assert_eq!(memory.histogram("mu.subscriber.lag"), Some((1, 0)));
    });
}

#[test]
fn lag_behind_the_head() {
    let memory = Memory::new();
    run(memory.subscriber(), async {
        let (journal, _stop, mut stream) = journal_stream(8).await;
        for n in 1..=3 {
            line(&journal, &format!("line {n}"));
        }
        for _ in 1..=3 {
            stream.next().await.unwrap().unwrap();
        }
        // The three events are 2, 1 and 0 behind the head.
        assert_eq!(memory.histogram("mu.subscriber.lag"), Some((3, 3)));
    });
}

/// A world on a fake game whose handler answers `go` from the game with `count` echoes.
fn burst(count: usize) -> (WorldSetup, mu_fake_server::FakeGame) {
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

#[test]
fn command_past_the_depth_limit() {
    let memory = Memory::new();
    run_paused(memory.subscriber(), async {
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
    });
}

#[test]
fn command_over_the_rate() {
    let memory = Memory::new();
    run_paused(memory.subscriber(), async {
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
    });
}

#[test]
fn commands_at_the_rate() {
    let memory = Memory::new();
    run_paused(memory.subscriber(), async {
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
    });
}

#[test]
fn world_without_transport_is_disconnected() {
    let memory = Memory::new();
    run_paused(memory.subscriber(), async {
        let resolved =
            ResolvedWorld::with_default_login(id("alpha/aria"), "env:PW".parse().unwrap());
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
    });
}

/// A portal with `alpha/aria` on `game`, with no login and the default reconnect, changed by
/// `change`.
fn world_on_game(game: &ScriptedGame, change: impl FnOnce(&mut ResolvedWorld)) -> Portal {
    let mut world = ResolvedWorld::new(id("alpha/aria"));
    change(&mut world);
    portal(
        vec![WorldSetup::new(world, None, Arc::new(game.clone()))],
        Vec::new(),
    )
}

#[test]
fn keepalive_lines_are_counted() {
    let memory = Memory::new();
    run_paused(memory.subscriber(), async {
        let game = ScriptedGame::new(Script::new());
        let portal = world_on_game(&game, |world| {
            let keepalive = Keepalive::new("IDLE", 60).unwrap();
            world.keepalive = keepalive.with_mode(KeepaliveMode::Interval);
        });
        portal.connect(&id("alpha/aria")).await.unwrap();
        wait_state(&portal, &id("alpha/aria"), WorldState::Connected).await;
        // The paused clock moves to 130 seconds, through the keepalives at 60 and 120.
        let until = tokio::time::timeout(Duration::from_secs(130), std::future::pending::<()>());
        assert!(until.await.is_err());
    });
    assert_eq!(
        memory.value("mu.keepalive.sent", &[("mu.world", "alpha/aria")]),
        Some(2)
    );
}

#[test]
fn gauge_shows_reconnecting() {
    let memory = Memory::new();
    run_paused(memory.subscriber(), async {
        let game = ScriptedGame::new(Script::new().close());
        let portal = world_on_game(&game, |world| {
            world.reconnect = Reconnect::Backoff(Backoff::new(1, 60, 30).unwrap());
        });
        portal.connect(&id("alpha/aria")).await.unwrap();
        wait_state(&portal, &id("alpha/aria"), WorldState::Reconnecting).await;
        let state = |name| {
            memory.value(
                "mu.world.state",
                &[("mu.world", "alpha/aria"), ("mu.world.state.name", name)],
            )
        };
        assert_eq!(
            (state("reconnecting"), state("connected")),
            (Some(1), Some(0))
        );
    });
}

#[test]
fn lost_connection_span_is_error() {
    let memory = Memory::new();
    run_paused(memory.subscriber(), async {
        let (portal, aria) = broken_world(bare_world(BrokenTransport::refusing_writes())).await;
        // The write of the line fails, so the connection is lost.
        let _ = portal.send(&aria, "hello").await;
        wait_state(&portal, &aria, WorldState::Disconnected).await;
    });
    let spans: Vec<_> = memory
        .finished_spans()
        .into_iter()
        .filter(|span| span.name == "world.connection")
        .collect();
    assert_eq!(spans.len(), 1, "{spans:?}");
    let end = spans[0]
        .attributes
        .iter()
        .find(|pair| pair.key.as_str() == "mu.connection.end")
        .map(|pair| pair.value.to_string());
    assert_eq!(end.as_deref(), Some("lost"));
    assert!(
        matches!(spans[0].status, opentelemetry::trace::Status::Error { .. }),
        "{:?}",
        spans[0].status
    );
}
