//! The telemetry of the engine under the layers of `mu server`, with in-memory exporters.

use mu_cli::telemetry::Providers;
use opentelemetry_sdk::metrics::InMemoryMetricExporter;
use opentelemetry_sdk::trace::{InMemorySpanExporter, SpanData};
use tracing_subscriber::layer::SubscriberExt as _;

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
    let _ = &memory.metrics;
}
