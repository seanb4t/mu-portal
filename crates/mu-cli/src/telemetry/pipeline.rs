//! The pipeline of the export: the exporters, the providers, the layers and the bounded stop.

use std::sync::mpsc;
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

use super::{Export, Protocol, Signal, Trust};

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

/// An error of the build of the export.
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    /// An exporter refused its configuration.
    #[error("the export: {0}")]
    Exporter(#[from] ExporterBuildError),
    /// The HTTP client cannot be built, or a certificate is bad.
    #[error("the export: {0}")]
    Client(#[from] reqwest::Error),
    /// The telemetry runtime or its thread cannot start.
    #[error("the export: {0}")]
    Runtime(#[source] std::io::Error),
    /// A gRPC signal has no telemetry runtime.
    #[error("the export: a gRPC signal has no telemetry runtime")]
    NoRuntime,
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

/// The providers of the export. Without a signal, a provider is absent.
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
    /// outside a tokio runtime. The gRPC exporters are built on the `mu-telemetry` runtime.
    ///
    /// # Errors
    ///
    /// A [`PipelineError`] of an exporter, of the HTTP client or of the runtime.
    pub fn start(export: &Export) -> Result<Self, PipelineError> {
        let grpc = [&export.traces, &export.metrics]
            .into_iter()
            .flatten()
            .any(|signal| signal.protocol == Protocol::Grpc);
        let runtime = grpc
            .then(TelemetryRuntime::start)
            .transpose()
            .map_err(PipelineError::Runtime)?;
        let handle = runtime.as_ref().map(|runtime| runtime.handle.clone());
        let spans = export
            .traces
            .as_ref()
            .map(|signal| span_exporter(signal, &export.trust, handle.as_ref()))
            .transpose()?;
        let metrics = export
            .metrics
            .as_ref()
            .map(|signal| metric_exporter(signal, &export.trust, handle.as_ref()))
            .transpose()?;
        let mut providers = Self::with_exporters(spans, metrics, &export.service_name);
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
    pub fn layers<S>(&self) -> impl Layer<S> + Send + Sync + use<S>
    where
        S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
    {
        let spans = self.tracer.as_ref().map(|provider| {
            tracing_opentelemetry::OpenTelemetryLayer::new(provider.tracer("mu"))
                .with_filter(filter_fn(|metadata| in_crates(metadata.target())))
        });
        let metrics = self.meter.as_ref().map(|provider| {
            tracing_opentelemetry::MetricsLayer::new(provider.clone())
                .with_filter(filter_fn(|metadata| metadata.target() == METRICS_TARGET))
        });
        tracing_subscriber::layer::Identity::new()
            .and_then(spans)
            .and_then(metrics)
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
    /// waits for both at most `bound`. A failed or late stop writes a `WARN` line. Then it sends
    /// the stop to the telemetry runtime, and joins its thread only if both stops finished in time.
    pub fn shutdown(self, bound: Duration) {
        let Self {
            tracer,
            meter,
            runtime,
        } = self;
        let deadline = Instant::now() + bound;
        let (done, finished) = mpsc::channel();
        let mut started = 0;
        let stops: [(&str, Option<Stop>); 2] = [
            (
                "traces",
                tracer.map(|tracer| Box::new(move || tracer.shutdown()) as _),
            ),
            (
                "metrics",
                meter.map(|meter| Box::new(move || meter.shutdown()) as _),
            ),
        ];
        for (signal, stop) in stops {
            let Some(stop) = stop else { continue };
            let done = done.clone();
            let spawned = thread::Builder::new()
                .name(format!("mu-telemetry-stop-{signal}"))
                .spawn(move || {
                    let _ = done.send((signal, stop()));
                });
            match spawned {
                Ok(_) => started += 1,
                Err(error) => tracing::warn!(signal, %error, "the stop of the export cannot start"),
            }
        }
        let mut in_time = true;
        for _ in 0..started {
            match finished.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                Ok((_, Ok(()))) => {}
                Ok((signal, Err(error))) => {
                    tracing::warn!(signal, %error, "the stop of the export failed");
                }
                Err(_) => {
                    in_time = false;
                    tracing::warn!(
                        "the export did not finish within {bound:?}; the rest of the telemetry is lost"
                    );
                    break;
                }
            }
        }
        if let Some(runtime) = runtime {
            let _ = runtime.stop.send(());
            if in_time && runtime.thread.join().is_err() {
                tracing::warn!("the thread of the telemetry runtime panicked");
            }
        }
    }
}

/// Whether `target` is one of [`CRATES`] or a module of one.
fn in_crates(target: &str) -> bool {
    CRATES.iter().any(|name| {
        target
            .strip_prefix(name)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with("::"))
    })
}

/// Whether the endpoint of `signal` has the scheme `https`, in any case.
fn is_https(signal: &Signal) -> bool {
    signal
        .endpoint
        .parse::<http::Uri>()
        .is_ok_and(|uri| uri.scheme() == Some(&http::uri::Scheme::HTTPS))
}

fn http_client(trust: &Trust) -> Result<reqwest::blocking::Client, PipelineError> {
    let certificates = trust
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

fn tls_config(trust: &Trust) -> ClientTlsConfig {
    let config = ClientTlsConfig::new().with_native_roots();
    if trust.pem.is_empty() {
        config
    } else {
        config.ca_certificate(Certificate::from_pem(&trust.pem))
    }
}

/// `builder` set for gRPC to the endpoint of `signal`, with TLS for `https`.
fn grpc<B: WithExportConfig + WithTonicConfig>(builder: B, signal: &Signal, trust: &Trust) -> B {
    let builder = builder
        .with_protocol(OtlpProtocol::Grpc)
        .with_timeout(EXPORT_TIMEOUT)
        .with_endpoint(signal.endpoint.clone());
    if is_https(signal) {
        builder.with_tls_config(tls_config(trust))
    } else {
        builder
    }
}

/// `builder` set for HTTP to the endpoint of `signal`.
fn http<B: WithExportConfig + WithHttpConfig>(
    builder: B,
    signal: &Signal,
    trust: &Trust,
) -> Result<B, PipelineError> {
    Ok(builder
        .with_http_client(http_client(trust)?)
        .with_protocol(OtlpProtocol::HttpBinary)
        .with_endpoint(signal.endpoint.clone()))
}

fn span_exporter(
    signal: &Signal,
    trust: &Trust,
    runtime: Option<&tokio::runtime::Handle>,
) -> Result<OtlpSpanExporter, PipelineError> {
    let builder = OtlpSpanExporter::builder();
    Ok(match (signal.protocol, runtime) {
        (Protocol::Grpc, Some(runtime)) => {
            let _entered = runtime.enter();
            grpc(builder.with_tonic(), signal, trust).build()?
        }
        (Protocol::HttpProtobuf, _) => http(builder.with_http(), signal, trust)?.build()?,
        (Protocol::Grpc, None) => return Err(PipelineError::NoRuntime),
    })
}

fn metric_exporter(
    signal: &Signal,
    trust: &Trust,
    runtime: Option<&tokio::runtime::Handle>,
) -> Result<MetricExporter, PipelineError> {
    let builder = MetricExporter::builder();
    Ok(match (signal.protocol, runtime) {
        (Protocol::Grpc, Some(runtime)) => {
            let _entered = runtime.enter();
            grpc(builder.with_tonic(), signal, trust).build()?
        }
        (Protocol::HttpProtobuf, _) => http(builder.with_http(), signal, trust)?.build()?,
        (Protocol::Grpc, None) => return Err(PipelineError::NoRuntime),
    })
}
