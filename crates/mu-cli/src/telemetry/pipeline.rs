//! The pipeline of the export: the exporters, the providers, the layers and the bounded stop.

use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use mu_core::METRICS_TARGET;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::tonic_types::transport::{Certificate, ClientTlsConfig};
use opentelemetry_otlp::{
    ExporterBuildError, MetricExporter, Protocol as OtlpProtocol, SpanExporter as OtlpSpanExporter,
    WithExportConfig, WithHttpConfig, WithTonicConfig,
};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::metrics::exporter::PushMetricExporter;
use opentelemetry_sdk::trace::{SdkTracerProvider, SpanExporter};
use tokio::sync::oneshot;
use tracing::Subscriber;
use tracing_subscriber::Layer;
use tracing_subscriber::filter::{LevelFilter, filter_fn};
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::registry::LookupSpan;

use super::{Authorities, Export, Protocol, Signal, SignalKind};

/// The longest wait for the export at the stop.
pub const STOP_BOUND: Duration = Duration::from_secs(5);

/// The longest wait for one export request.
const EXPORT_TIMEOUT: Duration = Duration::from_secs(10);

/// The crates whose spans and events reach the span layer. A target matches a crate when it is
/// the crate name or starts with the crate name and `::`.
pub const CRATES: [&str; 3] = ["mu_core", "mu_server", "mu_cli"];

/// The stop of one provider.
type Stop = Box<dyn FnOnce() -> OTelSdkResult + Send>;

/// The name of the thread of the telemetry runtime.
const RUNTIME_THREAD: &str = "mu-telemetry";

/// An error of the build of the export. The message holds the text of the inner error, so no
/// variant also exposes it as a source.
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    /// An exporter refused its configuration. The text names the setting, never its value.
    #[error("the export: {0}")]
    Exporter(String),
    /// The HTTP client cannot be built, or a certificate is bad.
    #[error("the export: {0}")]
    Client(reqwest::Error),
    /// The telemetry runtime or its thread cannot start.
    #[error("the export: {0}")]
    Runtime(std::io::Error),
}

impl From<ExporterBuildError> for PipelineError {
    fn from(error: ExporterBuildError) -> Self {
        Self::Exporter(match error {
            // The text is `<setting>: <reason>`, and the reason can quote the endpoint.
            ExporterBuildError::InvalidConfiguration(text) => match text.split_once(':') {
                Some((setting, _)) => format!("an exporter refused the setting `{setting}`"),
                None => "an exporter refused its configuration".to_owned(),
            },
            ExporterBuildError::InternalFailure(text) => {
                format!("an exporter cannot start: {text}")
            }
        })
    }
}

impl From<reqwest::Error> for PipelineError {
    fn from(error: reqwest::Error) -> Self {
        Self::Client(error)
    }
}

/// How the exporters of one signal reach the collector.
enum Transport {
    /// The blocking HTTP client.
    Http,
    /// tonic, on the telemetry runtime of this handle.
    Grpc(tokio::runtime::Handle),
}

/// How one stop of a provider ended.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// The provider flushed and stopped.
    Done,
    /// The provider reported this error.
    Failed(String),
    /// The helper thread cannot start, with this error.
    CannotStart(String),
    /// The helper thread ended without a result.
    Panicked,
    /// The bound passed first.
    Late,
}

/// The current-thread tokio runtime of the gRPC exporters, on its own thread.
struct TelemetryRuntime {
    handle: tokio::runtime::Handle,
    stop: oneshot::Sender<()>,
    thread: thread::JoinHandle<()>,
}

impl TelemetryRuntime {
    fn start() -> std::io::Result<Self> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let handle = runtime.handle().clone();
        let (stop, stopped) = oneshot::channel::<()>();
        let thread = thread::Builder::new()
            .name(RUNTIME_THREAD.to_owned())
            .spawn(move || {
                runtime.block_on(async {
                    let _ = stopped.await;
                });
            })?;
        Ok(Self {
            handle,
            stop,
            thread,
        })
    }
}

/// The providers of the export. Without a signal, a provider is absent. A drop stops the export
/// within [`STOP_BOUND`], and [`Providers::shutdown`] stops it within another bound.
pub struct Providers {
    tracer: Option<SdkTracerProvider>,
    meter: Option<SdkMeterProvider>,
    runtime: Option<TelemetryRuntime>,
}

impl Providers {
    /// No export.
    #[must_use]
    pub fn none() -> Self {
        Self {
            tracer: None,
            meter: None,
            runtime: None,
        }
    }

    /// The providers of `spans` and `metrics`, with the resource of `service_name`.
    #[must_use]
    pub fn with_exporters<S, M>(spans: Option<S>, metrics: Option<M>, service_name: &str) -> Self
    where
        S: SpanExporter + 'static,
        M: PushMetricExporter,
    {
        let resource = Resource::builder()
            .with_service_name(service_name.to_owned())
            .build();
        Self {
            tracer: spans.map(|exporter| {
                SdkTracerProvider::builder()
                    .with_batch_exporter(exporter)
                    .with_resource(resource.clone())
                    .build()
            }),
            meter: metrics.map(|exporter| {
                SdkMeterProvider::builder()
                    .with_periodic_exporter(exporter)
                    .with_resource(resource)
                    .build()
            }),
            runtime: None,
        }
    }

    /// The providers of `export`. The HTTP exporters are built here, so the caller must be
    /// outside a tokio runtime. The gRPC exporters are built in the context of the
    /// `mu-telemetry` runtime, which starts with the first gRPC signal, and their tasks run on
    /// its thread.
    ///
    /// # Errors
    ///
    /// A [`PipelineError`] of an exporter, of the HTTP client or of the runtime.
    pub fn start(export: &Export) -> Result<Self, PipelineError> {
        let mut runtime = None;
        let authorities = export.authorities();
        let spans = export
            .traces()
            .map(|signal| {
                let transport = transport(signal.protocol(), &mut runtime)?;
                span_exporter(signal, authorities, &transport)
            })
            .transpose()?;
        let metrics = export
            .metrics()
            .map(|signal| {
                let transport = transport(signal.protocol(), &mut runtime)?;
                metric_exporter(signal, authorities, &transport)
            })
            .transpose()?;
        let mut providers = Self::with_exporters(spans, metrics, export.service_name());
        providers.runtime = runtime;
        Ok(providers)
    }

    /// The tracer provider, if the traces have an endpoint.
    #[must_use]
    pub fn tracer_provider(&self) -> Option<&SdkTracerProvider> {
        self.tracer.as_ref()
    }

    /// The meter provider, if the metrics have an endpoint.
    #[must_use]
    pub fn meter_provider(&self) -> Option<&SdkMeterProvider> {
        self.meter.as_ref()
    }

    /// The two OpenTelemetry layers. The span layer takes the targets of [`CRATES`], and the
    /// metrics layer takes only [`mu_core::METRICS_TARGET`].
    #[must_use]
    pub fn layers<S>(&self) -> Vec<Box<dyn Layer<S> + Send + Sync>>
    where
        S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
    {
        // Only the layers that are present: an absent layer in the stack still enables its
        // targets, and `event_stream` then reads the lag with no metrics layer.
        let mut layers = Vec::new();
        if let Some(provider) = &self.tracer {
            layers.push(
                tracing_opentelemetry::OpenTelemetryLayer::new(provider.tracer("mu"))
                    .with_filter(filter_fn(|metadata| in_crates(metadata.target())))
                    .boxed(),
            );
        }
        if let Some(provider) = &self.meter {
            layers.push(
                tracing_opentelemetry::MetricsLayer::new(provider.clone())
                    .with_filter(filter_fn(|metadata| metadata.target() == METRICS_TARGET))
                    .boxed(),
            );
        }
        layers
    }

    /// The subscriber of `mu server`: the stderr lines at `WARN`, and the layers of the export.
    pub fn subscriber(&self) -> impl Subscriber + Send + Sync + use<> {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(std::io::stderr)
                    .with_filter(LevelFilter::WARN),
            )
            .with(self.layers())
    }

    /// Flushes and stops the two providers at the same time, each on its own helper thread, and
    /// waits at most `bound` for the stops that start. A stop that fails, cannot start, panics or
    /// is late writes a `WARN` line, and the line of a late stop names its signals. Then it sends
    /// the stop to the telemetry runtime, and joins its thread only if no stop is late.
    pub fn shutdown(mut self, bound: Duration) {
        self.stop(bound);
    }

    fn stop(&mut self, bound: Duration) {
        let tracer = self.tracer.take();
        let meter = self.meter.take();
        // Each helper thread gets a clone, so a thread that cannot start drops no last reference
        // here: the drop of the last reference stops a provider with no bound.
        let mut stops: Vec<(SignalKind, Stop)> = Vec::new();
        if let Some(tracer) = tracer.clone() {
            stops.push((SignalKind::Traces, Box::new(move || tracer.shutdown())));
        }
        if let Some(meter) = meter.clone() {
            stops.push((SignalKind::Metrics, Box::new(move || meter.shutdown())));
        }
        let outcomes = stop_all(stops, bound);
        let mut late = Vec::new();
        for (kind, outcome) in &outcomes {
            let signal = kind.name();
            match outcome {
                Outcome::Done => {}
                Outcome::Failed(error) => {
                    tracing::warn!(signal, %error, "the stop of the export failed");
                }
                Outcome::CannotStart(error) => {
                    tracing::warn!(signal, %error, "the stop of the export cannot start");
                }
                Outcome::Panicked => tracing::warn!(signal, "the stop of the export panicked"),
                Outcome::Late => late.push(signal),
            }
        }
        if !late.is_empty() {
            tracing::warn!(
                signals = %late.join(", "),
                "the export did not finish within {bound:?}; the rest of the telemetry is lost"
            );
        }
        let cannot_start = |kind: SignalKind| {
            outcomes.iter().any(|(signal, outcome)| {
                *signal == kind && matches!(outcome, Outcome::CannotStart(_))
            })
        };
        // A provider whose stop cannot start is never dropped, so its drop cannot block.
        if cannot_start(SignalKind::Traces) {
            std::mem::forget(tracer);
        }
        if cannot_start(SignalKind::Metrics) {
            std::mem::forget(meter);
        }
        if let Some(runtime) = self.runtime.take() {
            let _ = runtime.stop.send(());
            if late.is_empty() && runtime.thread.join().is_err() {
                tracing::warn!("the thread of the telemetry runtime panicked");
            }
        }
    }
}

impl Drop for Providers {
    fn drop(&mut self) {
        self.stop(STOP_BOUND);
    }
}

/// Runs each stop on its own helper thread, and waits at most `bound` for the stops that start.
fn stop_all(stops: Vec<(SignalKind, Stop)>, bound: Duration) -> Vec<(SignalKind, Outcome)> {
    let deadline = Instant::now() + bound;
    let (done, finished) = mpsc::channel();
    let mut outcomes = Vec::new();
    let mut pending = Vec::new();
    for (signal, stop) in stops {
        let done = done.clone();
        let spawned = thread::Builder::new()
            .name(format!("mu-telemetry-stop-{}", signal.name()))
            .spawn(move || {
                let _ = done.send((signal, stop()));
            });
        match spawned {
            Ok(_) => pending.push(signal),
            Err(error) => outcomes.push((signal, Outcome::CannotStart(error.to_string()))),
        }
    }
    // The channel disconnects when each helper thread ended, so a thread that panics before it
    // sends ends the wait early.
    drop(done);
    while !pending.is_empty() {
        match finished.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
            Ok((signal, result)) => {
                pending.retain(|kind| *kind != signal);
                let outcome = match result {
                    Ok(()) => Outcome::Done,
                    Err(error) => Outcome::Failed(error.to_string()),
                };
                outcomes.push((signal, outcome));
            }
            Err(RecvTimeoutError::Timeout) => {
                outcomes.extend(pending.drain(..).map(|signal| (signal, Outcome::Late)));
            }
            Err(RecvTimeoutError::Disconnected) => {
                outcomes.extend(pending.drain(..).map(|signal| (signal, Outcome::Panicked)));
            }
        }
    }
    outcomes
}

/// The transport of `protocol`. The first gRPC signal starts the telemetry runtime.
fn transport(
    protocol: Protocol,
    runtime: &mut Option<TelemetryRuntime>,
) -> Result<Transport, PipelineError> {
    Ok(match protocol {
        Protocol::HttpProtobuf => Transport::Http,
        Protocol::Grpc => {
            let runtime = match runtime {
                Some(runtime) => runtime,
                None => runtime.insert(TelemetryRuntime::start().map_err(PipelineError::Runtime)?),
            };
            Transport::Grpc(runtime.handle.clone())
        }
    })
}

/// Whether `target` is one of [`CRATES`] or a module of one.
fn in_crates(target: &str) -> bool {
    CRATES.iter().any(|name| {
        target
            .strip_prefix(name)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with("::"))
    })
}

fn http_client(authorities: &Authorities) -> Result<reqwest::blocking::Client, PipelineError> {
    let certificates = authorities
        .certificates()
        .iter()
        .map(|der| reqwest::Certificate::from_der(der))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(reqwest::blocking::Client::builder()
        .timeout(EXPORT_TIMEOUT)
        .redirect(reqwest::redirect::Policy::none())
        .tls_certs_merge(certificates)
        .build()?)
}

fn tls_config(authorities: &Authorities) -> ClientTlsConfig {
    let config = ClientTlsConfig::new().with_native_roots();
    if authorities.pem().is_empty() {
        config
    } else {
        config.ca_certificate(Certificate::from_pem(authorities.pem()))
    }
}

/// `builder` set for gRPC to the endpoint of `signal`, with TLS for `https`.
fn grpc<B: WithExportConfig + WithTonicConfig>(
    builder: B,
    signal: &Signal,
    authorities: &Authorities,
) -> B {
    let builder = builder
        .with_protocol(OtlpProtocol::Grpc)
        .with_timeout(EXPORT_TIMEOUT)
        .with_endpoint(signal.endpoint());
    if signal.is_https() {
        builder.with_tls_config(tls_config(authorities))
    } else {
        builder
    }
}

/// `builder` set for HTTP to the endpoint of `signal`. The timeout in code keeps the timeout
/// variables of OpenTelemetry from setting the retry deadline.
fn http<B: WithExportConfig + WithHttpConfig>(
    builder: B,
    signal: &Signal,
    authorities: &Authorities,
) -> Result<B, PipelineError> {
    Ok(builder
        .with_http_client(http_client(authorities)?)
        .with_protocol(OtlpProtocol::HttpBinary)
        .with_timeout(EXPORT_TIMEOUT)
        .with_endpoint(signal.endpoint()))
}

fn span_exporter(
    signal: &Signal,
    authorities: &Authorities,
    transport: &Transport,
) -> Result<OtlpSpanExporter, PipelineError> {
    let builder = OtlpSpanExporter::builder();
    Ok(match transport {
        Transport::Grpc(runtime) => {
            let _entered = runtime.enter();
            grpc(builder.with_tonic(), signal, authorities).build()?
        }
        Transport::Http => http(builder.with_http(), signal, authorities)?.build()?,
    })
}

fn metric_exporter(
    signal: &Signal,
    authorities: &Authorities,
    transport: &Transport,
) -> Result<MetricExporter, PipelineError> {
    let builder = MetricExporter::builder();
    Ok(match transport {
        Transport::Grpc(runtime) => {
            let _entered = runtime.enter();
            grpc(builder.with_tonic(), signal, authorities).build()?
        }
        Transport::Http => http(builder.with_http(), signal, authorities)?.build()?,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use opentelemetry_sdk::error::OTelSdkError;

    use super::*;

    /// A bound that no stop in these tests reaches unless the wait is wrong.
    const LONG: Duration = Duration::from_secs(60);

    fn stop(result: OTelSdkResult) -> Stop {
        Box::new(move || result)
    }

    #[test]
    fn done_and_failed_stops_are_reported() {
        let outcomes = stop_all(
            vec![
                (SignalKind::Traces, stop(Ok(()))),
                (
                    SignalKind::Metrics,
                    stop(Err(OTelSdkError::InternalFailure("x".into()))),
                ),
            ],
            LONG,
        );
        assert!(
            outcomes.contains(&(SignalKind::Traces, Outcome::Done)),
            "{outcomes:?}"
        );
        assert!(
            outcomes
                .iter()
                .any(|(signal, outcome)| *signal == SignalKind::Metrics
                    && matches!(outcome, Outcome::Failed(_))),
            "{outcomes:?}"
        );
    }

    #[test]
    fn panicked_stop_is_reported_without_the_bound() {
        let outcomes = stop_all(
            vec![
                (
                    SignalKind::Traces,
                    Box::new(|| panic!("a stop that panics")),
                ),
                (SignalKind::Metrics, stop(Ok(()))),
            ],
            LONG,
        );
        assert!(
            outcomes.contains(&(SignalKind::Traces, Outcome::Panicked)),
            "{outcomes:?}"
        );
        assert!(
            outcomes.contains(&(SignalKind::Metrics, Outcome::Done)),
            "{outcomes:?}"
        );
    }

    #[test]
    fn late_stop_is_named() {
        let (open, gate) = mpsc::channel::<()>();
        let outcomes = stop_all(
            vec![(
                SignalKind::Metrics,
                Box::new(move || {
                    let _ = gate.recv();
                    Ok(())
                }),
            )],
            Duration::ZERO,
        );
        drop(open);
        assert_eq!(outcomes, vec![(SignalKind::Metrics, Outcome::Late)]);
    }

    #[test]
    fn exporter_error_names_the_setting_and_not_the_value() {
        let error = PipelineError::from(ExporterBuildError::InvalidConfiguration(
            "endpoint: invalid endpoint 'http://u:hunter2@h': bad".to_owned(),
        ))
        .to_string();
        assert!(error.contains("`endpoint`"), "{error}");
        assert!(!error.contains("hunter2"), "{error}");
    }
}
