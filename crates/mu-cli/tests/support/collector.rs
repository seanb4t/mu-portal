//! A fake OTLP collector on `127.0.0.1`, which records each request that it gets.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::Bytes;
use axum::http::{HeaderMap, Method, StatusCode, Uri, header};
use axum::response::IntoResponse;
use opentelemetry_proto::tonic::collector::metrics::v1::metrics_service_server::{
    MetricsService, MetricsServiceServer,
};
use opentelemetry_proto::tonic::collector::metrics::v1::{
    ExportMetricsServiceRequest, ExportMetricsServiceResponse,
};
use opentelemetry_proto::tonic::collector::trace::v1::trace_service_server::{
    TraceService, TraceServiceServer,
};
use opentelemetry_proto::tonic::collector::trace::v1::{
    ExportTraceServiceRequest, ExportTraceServiceResponse,
};
use prost::Message as _;
use tempfile::TempDir;
use tokio::task::JoinHandle;

/// The HTTP paths under this prefix answer with a redirect to [`ELSEWHERE`].
pub const REDIRECT: &str = "/redirect";

/// The target of each redirect.
pub const ELSEWHERE: &str = "/elsewhere";

/// The path of the gRPC call of the traces.
pub const TRACE_EXPORT: &str = "/opentelemetry.proto.collector.trace.v1.TraceService/Export";
/// The path of the gRPC call of the metrics.
pub const METRICS_EXPORT: &str = "/opentelemetry.proto.collector.metrics.v1.MetricsService/Export";

/// How the collector serves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// OTLP/HTTP.
    Http,
    /// OTLP/HTTP over TLS, with a certificate of the test authority.
    Https,
    /// OTLP/gRPC.
    Grpc,
    /// OTLP/gRPC over TLS, with a certificate of the test authority.
    GrpcTls,
    /// Accepts each connection, and never reads or answers.
    Silent,
}

/// One request that the collector got.
#[derive(Clone, Debug)]
pub struct Request {
    /// `POST` for HTTP, `gRPC` for a gRPC call.
    pub method: String,
    /// The path of the URL, or of the gRPC call.
    pub path: String,
    /// The content type of an HTTP request. Empty for gRPC.
    pub content_type: String,
    /// The protobuf body.
    pub body: Vec<u8>,
}

type Log = Arc<Mutex<Vec<Request>>>;

/// A running fake collector. The drop stops it.
pub struct Collector {
    address: SocketAddr,
    mode: Mode,
    log: Log,
    dir: TempDir,
    task: JoinHandle<()>,
}

impl Drop for Collector {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// The test authority and the certificate of `127.0.0.1` that it signs, in PEM.
struct Pki {
    authority: String,
    certificate: String,
    key: String,
}

fn pki() -> Pki {
    let authority_key = rcgen::KeyPair::generate().unwrap();
    let mut authority = rcgen::CertificateParams::new(Vec::new()).unwrap();
    authority.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    authority
        .distinguished_name
        .push(rcgen::DnType::CommonName, "mu test authority");
    let authority_pem = authority.self_signed(&authority_key).unwrap().pem();
    let issuer = rcgen::Issuer::new(authority, authority_key);
    let key = rcgen::KeyPair::generate().unwrap();
    let mut leaf = rcgen::CertificateParams::new(Vec::new()).unwrap();
    leaf.subject_alt_names = vec![rcgen::SanType::IpAddress([127, 0, 0, 1].into())];
    let certificate = leaf.signed_by(&key, &issuer).unwrap().pem();
    Pki {
        authority: authority_pem,
        certificate,
        key: key.serialize_pem(),
    }
}

impl Collector {
    /// Starts a collector in `mode` on a free port.
    pub async fn start(mode: Mode) -> Self {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let dir = tempfile::tempdir().unwrap();
        let pki = pki();
        std::fs::write(dir.path().join("ca.pem"), &pki.authority).unwrap();
        let log = Log::default();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = match mode {
            Mode::Http => {
                let app = http_app(Arc::clone(&log));
                tokio::spawn(async move {
                    axum::serve(listener, app).await.unwrap();
                })
            }
            Mode::Https => {
                let app = http_app(Arc::clone(&log));
                let config = axum_server::tls_rustls::RustlsConfig::from_pem(
                    pki.certificate.into_bytes(),
                    pki.key.into_bytes(),
                )
                .await
                .unwrap();
                let listener = listener.into_std().unwrap();
                let server = axum_server::tls_rustls::from_tcp_rustls(listener, config).unwrap();
                tokio::spawn(async move {
                    server.serve(app.into_make_service()).await.unwrap();
                })
            }
            Mode::Grpc | Mode::GrpcTls => {
                let mut builder = tonic::transport::Server::builder();
                if mode == Mode::GrpcTls {
                    let identity = tonic::transport::Identity::from_pem(&pki.certificate, &pki.key);
                    builder = builder
                        .tls_config(tonic::transport::ServerTlsConfig::new().identity(identity))
                        .unwrap();
                }
                let router = builder
                    .add_service(TraceServiceServer::new(Grpc(Arc::clone(&log))))
                    .add_service(MetricsServiceServer::new(Grpc(Arc::clone(&log))));
                let incoming = tonic::transport::server::TcpIncoming::from(listener);
                tokio::spawn(async move {
                    router.serve_with_incoming(incoming).await.unwrap();
                })
            }
            Mode::Silent => tokio::spawn(async move {
                let mut held = Vec::new();
                while let Ok((stream, _)) = listener.accept().await {
                    held.push(stream);
                }
            }),
        };
        Self {
            address,
            mode,
            log,
            dir,
            task,
        }
    }

    /// The base URL: `https` in a TLS mode, `http` otherwise.
    pub fn url(&self) -> String {
        let scheme = match self.mode {
            Mode::Https | Mode::GrpcTls => "https",
            _ => "http",
        };
        format!("{scheme}://{}", self.address)
    }

    /// The PEM file of the test authority.
    pub fn authority(&self) -> PathBuf {
        self.dir.path().join("ca.pem")
    }

    /// The directory of the files of the collector.
    pub fn dir(&self) -> &Path {
        self.dir.path()
    }

    /// Each request so far.
    ///
    /// # Panics
    ///
    /// If the collector stopped serving: its task ends only on an error.
    pub fn requests(&self) -> Vec<Request> {
        assert!(
            !self.task.is_finished(),
            "the fake collector stopped serving"
        );
        self.log.lock().unwrap().clone()
    }

    /// The paths of the requests so far.
    pub fn paths(&self) -> Vec<String> {
        self.requests()
            .into_iter()
            .map(|request| request.path)
            .collect()
    }

    /// Whether a request went to `path`.
    pub fn got(&self, path: &str) -> bool {
        self.requests().iter().any(|request| request.path == path)
    }

    /// The trace requests, at a path that ends in `traces` or the gRPC call, decoded.
    pub fn traces(&self) -> Vec<ExportTraceServiceRequest> {
        self.requests()
            .iter()
            .filter(|request| request.path.ends_with("traces") || request.path == TRACE_EXPORT)
            .map(|request| ExportTraceServiceRequest::decode(request.body.as_slice()).unwrap())
            .collect()
    }

    /// The metric requests, at a path that ends in `metrics` or the gRPC call, decoded.
    pub fn metrics(&self) -> Vec<ExportMetricsServiceRequest> {
        self.requests()
            .iter()
            .filter(|request| request.path.ends_with("metrics") || request.path == METRICS_EXPORT)
            .map(|request| ExportMetricsServiceRequest::decode(request.body.as_slice()).unwrap())
            .collect()
    }
}

fn http_app(log: Log) -> Router {
    Router::new().fallback(
        move |method: Method, uri: Uri, headers: HeaderMap, body: Bytes| {
            let log = Arc::clone(&log);
            async move {
                let content_type = headers
                    .get(header::CONTENT_TYPE)
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or_default()
                    .to_owned();
                log.lock().unwrap().push(Request {
                    method: method.to_string(),
                    path: uri.path().to_owned(),
                    content_type,
                    body: body.to_vec(),
                });
                if uri.path().starts_with(REDIRECT) {
                    return (
                        StatusCode::TEMPORARY_REDIRECT,
                        [(header::LOCATION, ELSEWHERE)],
                    )
                        .into_response();
                }
                (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "application/x-protobuf")],
                    Vec::<u8>::new(),
                )
                    .into_response()
            }
        },
    )
}

struct Grpc(Log);

impl Grpc {
    fn record(&self, path: &str, body: Vec<u8>) {
        self.0.lock().unwrap().push(Request {
            method: "gRPC".to_owned(),
            path: path.to_owned(),
            content_type: String::new(),
            body,
        });
    }
}

#[tonic::async_trait]
impl TraceService for Grpc {
    async fn export(
        &self,
        request: tonic::Request<ExportTraceServiceRequest>,
    ) -> Result<tonic::Response<ExportTraceServiceResponse>, tonic::Status> {
        self.record(TRACE_EXPORT, request.into_inner().encode_to_vec());
        Ok(tonic::Response::new(ExportTraceServiceResponse::default()))
    }
}

#[tonic::async_trait]
impl MetricsService for Grpc {
    async fn export(
        &self,
        request: tonic::Request<ExportMetricsServiceRequest>,
    ) -> Result<tonic::Response<ExportMetricsServiceResponse>, tonic::Status> {
        self.record(METRICS_EXPORT, request.into_inner().encode_to_vec());
        Ok(tonic::Response::new(ExportMetricsServiceResponse::default()))
    }
}
