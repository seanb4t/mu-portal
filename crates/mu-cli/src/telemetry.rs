//! The telemetry of `mu server`: where it sends traces and metrics, and its subscriber.

use std::fmt;
use std::path::Path;

use rustls::pki_types::CertificateDer;
use rustls::pki_types::pem::PemObject as _;

use crate::server_config::{SERVER_FILE, TelemetryTable};

mod pipeline;

pub use pipeline::{PipelineError, Providers, STOP_BOUND, TARGETS};

/// Turns the export off with the value `true`, in any case.
pub const SDK_DISABLED: &str = "OTEL_SDK_DISABLED";
/// The base URL of both signals.
pub const ENDPOINT: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";
/// The full URL of the traces.
pub const TRACES_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT";
/// The full URL of the metrics.
pub const METRICS_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_METRICS_ENDPOINT";
/// The protocol of both signals.
pub const PROTOCOL: &str = "OTEL_EXPORTER_OTLP_PROTOCOL";
/// The protocol of the traces.
pub const TRACES_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_TRACES_PROTOCOL";
/// The protocol of the metrics.
pub const METRICS_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_METRICS_PROTOCOL";
/// A PEM file of the authorities to trust beside the roots of the system.
pub const CERTIFICATE: &str = "OTEL_EXPORTER_OTLP_CERTIFICATE";
/// The value of `service.name`.
pub const SERVICE_NAME: &str = "OTEL_SERVICE_NAME";

/// The service name without `OTEL_SERVICE_NAME`.
pub const DEFAULT_SERVICE_NAME: &str = "mu";

/// The protocol of one signal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Protocol {
    /// `http/protobuf`: a `POST` of protobuf to the URL of the signal.
    HttpProtobuf,
    /// `grpc`: the OTLP gRPC services at the endpoint.
    Grpc,
}

/// One of the two signals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignalKind {
    /// The spans.
    Traces,
    /// The metrics.
    Metrics,
}

impl SignalKind {
    /// The path that a base URL gets for this signal over `http/protobuf`.
    #[must_use]
    pub fn path(self) -> &'static str {
        match self {
            Self::Traces => "/v1/traces",
            Self::Metrics => "/v1/metrics",
        }
    }

    fn endpoint_variable(self) -> &'static str {
        match self {
            Self::Traces => TRACES_ENDPOINT,
            Self::Metrics => METRICS_ENDPOINT,
        }
    }

    fn protocol_variable(self) -> &'static str {
        match self {
            Self::Traces => TRACES_PROTOCOL,
            Self::Metrics => METRICS_PROTOCOL,
        }
    }
}

/// Where one signal goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signal {
    /// The full URL, for `http/protobuf`, or the address of the services, for `grpc`.
    pub endpoint: String,
    /// The protocol.
    pub protocol: Protocol,
}

/// The authorities that the export trusts beside the roots of the system.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Trust {
    pem: Vec<u8>,
    certificates: Vec<CertificateDer<'static>>,
}

impl Trust {
    /// The certificates of the file, in DER. Empty without a file.
    #[must_use]
    pub fn certificates(&self) -> &[CertificateDer<'static>] {
        &self.certificates
    }
}

/// The export of the server. Without it, the server exports nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Export {
    /// The traces, if they have an endpoint.
    pub traces: Option<Signal>,
    /// The metrics, if they have an endpoint.
    pub metrics: Option<Signal>,
    /// The value of `service.name`.
    pub service_name: String,
    /// The authorities of the certificate file.
    pub trust: Trust,
}

/// Where a bad value came from: a variable, or a field of `[telemetry]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// A variable of the environment.
    Variable(&'static str),
    /// A field of `[telemetry]` in `server.toml`.
    Field(&'static str),
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Variable(name) => f.write_str(name),
            Self::Field(name) => write!(f, "{SERVER_FILE}: telemetry.{name}"),
        }
    }
}

/// A bad value of the telemetry: its source, and what is wrong.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{place}: {message}")]
pub struct SettingError {
    place: Source,
    message: String,
}

impl SettingError {
    fn new(place: Source, message: impl Into<String>) -> Self {
        Self {
            place,
            message: message.into(),
        }
    }

    /// The source of the bad value.
    #[must_use]
    pub fn place(&self) -> Source {
        self.place
    }
}

/// Picks the export from the variables, which `var` reads, and the table of `server.toml`. An
/// empty variable counts as unset. `None` means no export.
///
/// # Errors
///
/// A [`SettingError`] that names the variable or the field.
pub fn export(
    var: impl Fn(&str) -> Option<String>,
    file: Option<&TelemetryTable>,
) -> Result<Option<Export>, SettingError> {
    let var = |name: &str| var(name).filter(|value| !value.is_empty());
    if var(SDK_DISABLED).is_some_and(|value| value.eq_ignore_ascii_case("true")) {
        return Ok(None);
    }
    let file = file.cloned().unwrap_or_default();
    let base_variable = var(ENDPOINT);
    let base = base_variable.as_deref().or(file.endpoint.as_deref());
    // The field is checked even when the variable wins over it.
    for (value, place) in [
        (base_variable.as_deref(), Source::Variable(ENDPOINT)),
        (file.endpoint.as_deref(), Source::Field("endpoint")),
    ] {
        if let Some(value) = value {
            check_endpoint(value, true).map_err(|message| SettingError::new(place, message))?;
        }
    }
    let mut own = [None, None];
    for (slot, kind) in own
        .iter_mut()
        .zip([SignalKind::Traces, SignalKind::Metrics])
    {
        let name = kind.endpoint_variable();
        if let Some(value) = var(name) {
            check_endpoint(&value, false)
                .map_err(|message| SettingError::new(Source::Variable(name), message))?;
            *slot = Some(value);
        }
    }
    let [own_traces, own_metrics] = own;
    if base.is_none() && own_traces.is_none() && own_metrics.is_none() {
        return Ok(None);
    }

    let field_protocol = file
        .protocol
        .as_deref()
        .map(|value| parse_protocol(value, Source::Field("protocol")))
        .transpose()?;
    let all_protocol = var(PROTOCOL)
        .map(|value| parse_protocol(&value, Source::Variable(PROTOCOL)))
        .transpose()?;
    let mut signals = [None, None];
    for ((slot, kind), own) in signals
        .iter_mut()
        .zip([SignalKind::Traces, SignalKind::Metrics])
        .zip([own_traces, own_metrics])
    {
        let name = kind.protocol_variable();
        let protocol = var(name)
            .map(|value| parse_protocol(&value, Source::Variable(name)))
            .transpose()?
            .or(all_protocol)
            .or(field_protocol)
            .unwrap_or(Protocol::HttpProtobuf);
        let endpoint = own.or_else(|| base.map(|base| join(base, kind, protocol)));
        *slot = endpoint.map(|endpoint| Signal { endpoint, protocol });
    }
    let [traces, metrics] = signals;

    let certificate = var(CERTIFICATE)
        .map(|path| (path, Source::Variable(CERTIFICATE)))
        .or_else(|| {
            file.certificate
                .as_ref()
                .map(|path| (path.display().to_string(), Source::Field("certificate")))
        });
    let trust = match certificate {
        Some((path, place)) => {
            read_trust(Path::new(&path)).map_err(|message| SettingError::new(place, message))?
        }
        None => Trust::default(),
    };
    let service_name = var(SERVICE_NAME).unwrap_or_else(|| DEFAULT_SERVICE_NAME.to_owned());
    Ok(Some(Export {
        traces,
        metrics,
        service_name,
        trust,
    }))
}

/// Checks that `value` is a URL with the scheme `http` or `https` and a host. A `base` URL can
/// hold no query and no fragment, because a path follows it. The message never holds the value,
/// because a URL can hold a password.
///
/// # Errors
///
/// What is wrong with the value.
pub fn check_endpoint(value: &str, base: bool) -> Result<(), String> {
    if value.contains('#') {
        return Err("a URL of the export can hold no fragment".to_owned());
    }
    let uri: http::Uri = value.parse().map_err(|_| "is not a URL".to_owned())?;
    match uri.scheme_str() {
        Some("http" | "https") => {}
        Some(scheme) => return Err(format!("the scheme `{scheme}` is not http or https")),
        None => return Err("has no scheme: write http:// or https://".to_owned()),
    }
    if uri.host().is_none_or(str::is_empty) {
        return Err("has no host".to_owned());
    }
    if base && uri.query().is_some() {
        return Err("a base URL can hold no query".to_owned());
    }
    Ok(())
}

fn parse_protocol(value: &str, place: Source) -> Result<Protocol, SettingError> {
    match value {
        "http/protobuf" => Ok(Protocol::HttpProtobuf),
        "grpc" => Ok(Protocol::Grpc),
        other => Err(SettingError::new(
            place,
            format!("the protocol `{other}` is not http/protobuf or grpc"),
        )),
    }
}

/// The URL of `kind` under the base URL `base`.
fn join(base: &str, kind: SignalKind, protocol: Protocol) -> String {
    match protocol {
        Protocol::Grpc => base.to_owned(),
        Protocol::HttpProtobuf => format!("{}{}", base.trim_end_matches('/'), kind.path()),
    }
}

/// The certificates of the PEM file at `path`.
fn read_trust(path: &Path) -> Result<Trust, String> {
    let shown = path.display();
    let pem = std::fs::read(path).map_err(|error| format!("{shown}: {error}"))?;
    let certificates = CertificateDer::pem_slice_iter(&pem)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("{shown}: {error}"))?;
    if certificates.is_empty() {
        return Err(format!("{shown}: the file holds no PEM certificate"));
    }
    Ok(Trust { pem, certificates })
}

#[cfg(test)]
mod tests;
