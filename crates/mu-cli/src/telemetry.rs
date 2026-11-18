//! The telemetry of `mu server`: where it sends traces and metrics, and its subscriber.

use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};

use rustls::pki_types::CertificateDer;
use rustls::pki_types::pem::PemObject as _;

use crate::server_config::{SERVER_FILE, TelemetryTable};

mod pipeline;

pub use pipeline::{CRATES, PipelineError, Providers, STOP_BOUND};

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
    /// The two signals.
    pub const ALL: [Self; 2] = [Self::Traces, Self::Metrics];

    /// The lower-case name of the signal, for the log lines.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Traces => "traces",
            Self::Metrics => "metrics",
        }
    }

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
#[derive(Clone, PartialEq, Eq)]
pub struct Signal {
    endpoint: String,
    protocol: Protocol,
    https: bool,
}

impl Signal {
    /// The full URL, for `http/protobuf`, or the address of the services, for `grpc`.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// The protocol.
    #[must_use]
    pub fn protocol(&self) -> Protocol {
        self.protocol
    }

    /// Whether the endpoint has the scheme `https`, in any case.
    #[must_use]
    pub fn is_https(&self) -> bool {
        self.https
    }
}

/// The endpoint is hidden, because a URL can hold a password.
impl fmt::Debug for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Signal")
            .field("endpoint", &"<hidden>")
            .field("protocol", &self.protocol)
            .field("https", &self.https)
            .finish()
    }
}

/// The authorities that the export trusts beside the roots of the system. `pem` is the whole
/// file, and `certificates` holds its `CERTIFICATE` sections, each one a certificate that
/// rustls can parse.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Authorities {
    pem: Vec<u8>,
    certificates: Vec<CertificateDer<'static>>,
}

impl Authorities {
    /// The certificates of the file, in DER. Empty without a file.
    #[must_use]
    pub fn certificates(&self) -> &[CertificateDer<'static>] {
        &self.certificates
    }

    /// The text of the file, in PEM. Empty without a file.
    #[must_use]
    pub fn pem(&self) -> &[u8] {
        &self.pem
    }
}

/// The file is hidden, because it can also hold a private key.
impl fmt::Debug for Authorities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Authorities")
            .field("certificates", &self.certificates.len())
            .finish_non_exhaustive()
    }
}

/// The export of the server. Without it, the server exports nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Export {
    traces: Option<Signal>,
    metrics: Option<Signal>,
    service_name: String,
    authorities: Authorities,
}

impl Export {
    /// The traces, if they have an endpoint.
    #[must_use]
    pub fn traces(&self) -> Option<&Signal> {
        self.traces.as_ref()
    }

    /// The metrics, if they have an endpoint.
    #[must_use]
    pub fn metrics(&self) -> Option<&Signal> {
        self.metrics.as_ref()
    }

    /// The value of `service.name`.
    #[must_use]
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    /// The authorities of the certificate file.
    #[must_use]
    pub fn authorities(&self) -> &Authorities {
        &self.authorities
    }
}

/// A field of `[telemetry]` in `server.toml`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableField {
    /// `endpoint`.
    Endpoint,
    /// `protocol`.
    Protocol,
    /// `certificate`.
    Certificate,
}

impl TableField {
    /// The name of the field.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Endpoint => "endpoint",
            Self::Protocol => "protocol",
            Self::Certificate => "certificate",
        }
    }
}

/// Where a bad value came from: a variable, or a field of `[telemetry]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingSource {
    /// A variable of the environment.
    Variable(&'static str),
    /// A field of `[telemetry]` in `server.toml`.
    Field(TableField),
}

impl fmt::Display for SettingSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Variable(name) => f.write_str(name),
            Self::Field(field) => write!(f, "{SERVER_FILE}: telemetry.{}", field.name()),
        }
    }
}

/// A bad value of the telemetry: its source, and what is wrong.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{place}: {message}")]
pub struct SettingError {
    place: SettingSource,
    message: String,
}

impl SettingError {
    fn new(place: SettingSource, message: impl Into<String>) -> Self {
        Self {
            place,
            message: message.into(),
        }
    }

    /// The source of the bad value.
    #[must_use]
    pub fn place(&self) -> SettingSource {
        self.place
    }
}

/// What an endpoint is for: the base URL of both signals, or the full URL of one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EndpointKind {
    Base,
    Signal,
}

/// A checked endpoint, and its source.
struct Checked {
    value: String,
    uri: http::Uri,
    place: SettingSource,
}

/// Whether `OTEL_SDK_DISABLED`, which `var` reads, holds a value other than `true`, `false` or
/// empty, in any case. [`export`] ignores such a value.
#[must_use]
pub fn sdk_disabled_is_ignored(var: impl Fn(&str) -> Option<OsString>) -> bool {
    var(SDK_DISABLED).is_some_and(|value| {
        let value = value.to_string_lossy();
        !(value.is_empty()
            || value.eq_ignore_ascii_case("true")
            || value.eq_ignore_ascii_case("false"))
    })
}

/// Picks the export from the variables, which `var` reads, and the table of `server.toml`. An
/// empty variable counts as unset. A variable that the export reads must be UTF-8, except
/// `OTEL_EXPORTER_OTLP_CERTIFICATE`, which is a path. `None` means no export.
///
/// # Errors
///
/// A [`SettingError`] that names the variable or the field.
pub fn export(
    var: impl Fn(&str) -> Option<OsString>,
    file: &TelemetryTable,
) -> Result<Option<Export>, SettingError> {
    let var_os = |name: &str| var(name).filter(|value| !value.is_empty());
    let var = |name: &'static str| {
        var_os(name)
            .map(|value| {
                value.into_string().map_err(|_| {
                    SettingError::new(SettingSource::Variable(name), "the value is not UTF-8")
                })
            })
            .transpose()
    };
    if var(SDK_DISABLED)?.is_some_and(|value| value.eq_ignore_ascii_case("true")) {
        return Ok(None);
    }
    let checked = |value: String, kind, place| {
        check_endpoint(&value, kind)
            .map(|uri| Checked { value, uri, place })
            .map_err(|message| SettingError::new(place, message))
    };
    // The field is checked even when the variable wins over it. The gRPC rule of no path and no
    // query applies only to the endpoint that a signal uses, in `signal`.
    let base_variable = var(ENDPOINT)?
        .map(|value| checked(value, EndpointKind::Base, SettingSource::Variable(ENDPOINT)))
        .transpose()?;
    let base_field = file
        .endpoint
        .clone()
        .map(|value| {
            checked(
                value,
                EndpointKind::Base,
                SettingSource::Field(TableField::Endpoint),
            )
        })
        .transpose()?;
    let base = base_variable.or(base_field);
    let mut own = [None, None];
    for (slot, kind) in own.iter_mut().zip(SignalKind::ALL) {
        let name = kind.endpoint_variable();
        *slot = var(name)?
            .map(|value| checked(value, EndpointKind::Signal, SettingSource::Variable(name)))
            .transpose()?;
    }
    if base.is_none() && own.iter().all(Option::is_none) {
        return Ok(None);
    }

    let field_protocol = file
        .protocol
        .as_deref()
        .map(|value| parse_protocol(value, SettingSource::Field(TableField::Protocol)))
        .transpose()?;
    let all_protocol = var(PROTOCOL)?
        .map(|value| parse_protocol(&value, SettingSource::Variable(PROTOCOL)))
        .transpose()?;
    let mut signals = [None, None];
    for ((slot, kind), own) in signals.iter_mut().zip(SignalKind::ALL).zip(own) {
        let name = kind.protocol_variable();
        let protocol = var(name)?
            .map(|value| parse_protocol(&value, SettingSource::Variable(name)))
            .transpose()?
            .or(all_protocol)
            .or(field_protocol)
            .unwrap_or(Protocol::HttpProtobuf);
        *slot = signal(kind, protocol, own.as_ref(), base.as_ref())?;
    }
    let [traces, metrics] = signals;

    let certificate = var_os(CERTIFICATE)
        .map(|path| (PathBuf::from(path), SettingSource::Variable(CERTIFICATE)))
        .or_else(|| {
            file.certificate
                .clone()
                .map(|path| (path, SettingSource::Field(TableField::Certificate)))
        });
    let authorities = match certificate {
        Some((path, place)) => {
            read_authorities(&path).map_err(|message| SettingError::new(place, message))?
        }
        None => Authorities::default(),
    };
    let service_name = var(SERVICE_NAME)?.unwrap_or_else(|| DEFAULT_SERVICE_NAME.to_owned());
    Ok(Some(Export {
        traces,
        metrics,
        service_name,
        authorities,
    }))
}

/// The signal `kind` over `protocol`, from its own endpoint or else from the base URL. A gRPC
/// endpoint can hold no path and no query: tonic sends its scheme and its authority, and drops the
/// path and the query.
fn signal(
    kind: SignalKind,
    protocol: Protocol,
    own: Option<&Checked>,
    base: Option<&Checked>,
) -> Result<Option<Signal>, SettingError> {
    let Some(source) = own.or(base) else {
        return Ok(None);
    };
    if protocol == Protocol::Grpc
        && (!matches!(source.uri.path(), "" | "/") || source.uri.query().is_some())
    {
        return Err(SettingError::new(
            source.place,
            "a gRPC endpoint can hold no path and no query",
        ));
    }
    let endpoint = match own {
        Some(own) => own.value.clone(),
        None => join(&source.value, kind, protocol),
    };
    Ok(Some(Signal {
        endpoint,
        protocol,
        https: source.uri.scheme() == Some(&http::uri::Scheme::HTTPS),
    }))
}

/// Checks that `value` is a URL with the scheme `http` or `https` and a host, and returns it
/// parsed. No URL can hold a fragment, a user or a password. A base URL can also hold no query,
/// with either protocol. The message holds no more of the value than its scheme.
fn check_endpoint(value: &str, kind: EndpointKind) -> Result<http::Uri, String> {
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
    if uri
        .authority()
        .is_some_and(|authority| authority.as_str().contains('@'))
    {
        return Err(
            "a URL of the export can hold no user and no password: put a token in a header"
                .to_owned(),
        );
    }
    if kind == EndpointKind::Base && uri.query().is_some() {
        return Err("a base URL can hold no query".to_owned());
    }
    Ok(uri)
}

/// The protocol of `value`. The message holds no part of the value, which can be a URL that was
/// put in the wrong place.
fn parse_protocol(value: &str, place: SettingSource) -> Result<Protocol, SettingError> {
    match value {
        "http/protobuf" => Ok(Protocol::HttpProtobuf),
        "grpc" => Ok(Protocol::Grpc),
        _ => Err(SettingError::new(
            place,
            "the protocol is not http/protobuf or grpc",
        )),
    }
}

/// The URL of `kind` for `protocol`: for `http/protobuf`, the base URL `base` with the path of
/// the signal, and for `grpc`, `base` itself.
fn join(base: &str, kind: SignalKind, protocol: Protocol) -> String {
    match protocol {
        Protocol::Grpc => base.to_owned(),
        Protocol::HttpProtobuf => format!("{}{}", base.trim_end_matches('/'), kind.path()),
    }
}

/// The certificates of the PEM file at `path`. Each one must parse as a certificate, because
/// over gRPC a certificate that does not parse is dropped without an error.
fn read_authorities(path: &Path) -> Result<Authorities, String> {
    let shown = path.display();
    let pem = std::fs::read(path).map_err(|error| format!("{shown}: {error}"))?;
    let certificates = CertificateDer::pem_slice_iter(&pem)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("{shown}: {error}"))?;
    if certificates.is_empty() {
        return Err(format!("{shown}: the file holds no PEM certificate"));
    }
    let mut roots = rustls::RootCertStore::empty();
    for (number, certificate) in certificates.iter().enumerate() {
        roots.add(certificate.clone()).map_err(|error| {
            format!("{shown}: certificate {} is not valid: {error}", number + 1)
        })?;
    }
    Ok(Authorities { pem, certificates })
}

#[cfg(test)]
mod tests;
