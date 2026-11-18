//! The scenarios of the telemetry capability, run on the built binary.

use std::process::{Command, ExitStatus, Output};

use opentelemetry_proto::tonic::common::v1::KeyValue;
use opentelemetry_proto::tonic::common::v1::any_value::Value;
use opentelemetry_proto::tonic::metrics::v1::metric::Data;
use opentelemetry_proto::tonic::metrics::v1::number_data_point::Value as Number;
use opentelemetry_proto::tonic::metrics::v1::{Metric, NumberDataPoint};
use opentelemetry_proto::tonic::resource::v1::Resource;
use opentelemetry_proto::tonic::trace::v1::Span;
use opentelemetry_proto::tonic::trace::v1::status::StatusCode;
use rustix::process::Signal;

use collector::{Collector, METRICS_EXPORT, Mode, TRACE_EXPORT};
use support::process::{PW, SERVING, ServerBuilder, Session};
use support::{ConfigDir, fails_naming};

mod support;

#[path = "support/collector.rs"]
mod collector;

// The variables of the export.
const ENDPOINT: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";
const TRACES_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT";
const METRICS_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_METRICS_ENDPOINT";
const PROTOCOL: &str = "OTEL_EXPORTER_OTLP_PROTOCOL";
const METRICS_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_METRICS_PROTOCOL";
const CERTIFICATE: &str = "OTEL_EXPORTER_OTLP_CERTIFICATE";
const SDK_DISABLED: &str = "OTEL_SDK_DISABLED";
const SERVICE_NAME: &str = "OTEL_SERVICE_NAME";

/// The text of the line that a stop past the bound writes.
const LATE_STOP: &str = "did not finish within";

/// The text of the line that a failed stop writes.
const FAILED_STOP: &str = "the stop of the export failed";

/// A server with the fake world `alpha/aria`, and its configuration directory.
struct Setup {
    builder: ServerBuilder,
    config: ConfigDir,
}

impl Setup {
    /// A server whose `server.toml` holds `[telemetry]` with `lines`. No lines means no file.
    fn new(lines: &[String]) -> Self {
        let config = if lines.is_empty() {
            ConfigDir::empty()
        } else {
            ConfigDir::empty().file(
                "server.toml",
                &format!("[telemetry]\n{}\n", lines.join("\n")),
            )
        };
        let builder = ServerBuilder::new()
            .env("MU_CONFIG", Some(config.path()))
            .fake_worlds(&["alpha/aria"]);
        Self { builder, config }
    }

    /// A server whose `server.toml` names `url`.
    fn endpoint(url: &str) -> Self {
        Self::new(&[format!("endpoint = \"{url}\"")])
    }

    fn env(mut self, name: &str, value: &str) -> Self {
        self.builder = self.builder.env(name, Some(value));
        self
    }

    /// Writes `text` to the file at `path` in the configuration directory.
    fn file(mut self, path: &str, text: &str) -> Self {
        self.config = self.config.file(path, text);
        self
    }

    /// Starts the server, sends one line to `alpha/aria`, and stops it with SIGTERM.
    async fn one_send(self) -> Run {
        self.run(&["hello"]).await
    }

    /// Starts the server and waits for the greeting of `alpha/aria`.
    async fn start(self) -> Session {
        let mut session = Session::start(self.builder).await;
        session.welcome("alpha/aria").await;
        session
    }

    /// Starts the server, sends each of `lines` to `alpha/aria`, and stops it with SIGTERM.
    async fn run(self, lines: &[&str]) -> Run {
        let session = self.start().await;
        for line in lines {
            session.client.send("alpha/aria", line).await.unwrap();
        }
        stop(session.server).await
    }

    /// Runs the server to its exit, for a server that refuses to start.
    async fn output(self) -> Output {
        let _config = self.config;
        self.builder.output().await
    }
}

/// The end of a server: its exit status and its stderr.
struct Run {
    status: ExitStatus,
    stderr: Vec<String>,
}

async fn stop(mut server: support::process::ServerProcess) -> Run {
    server.signal(Signal::TERM);
    let status = server.wait_exit().await;
    server.wait_stderr(|_| false).await;
    Run {
        status,
        stderr: server.stderr(),
    }
}

/// Each span that `collector` got.
fn spans(collector: &Collector) -> Vec<Span> {
    collector
        .traces()
        .into_iter()
        .flat_map(|request| request.resource_spans)
        .flat_map(|spans| spans.scope_spans)
        .flat_map(|scope| scope.spans)
        .collect()
}

/// The spans named `name` that `collector` got.
fn spans_named(collector: &Collector, name: &str) -> Vec<Span> {
    spans(collector)
        .into_iter()
        .filter(|span| span.name == name)
        .collect()
}

/// The one span named `name` that `collector` got.
#[track_caller]
fn one_span(collector: &Collector, name: &str) -> Span {
    let mut found = spans_named(collector, name);
    let names: Vec<_> = spans(collector).into_iter().map(|span| span.name).collect();
    assert_eq!(found.len(), 1, "spans named {name}: {names:?}");
    found.remove(0)
}

/// The text form of the attribute `key` of `span`.
fn attribute(span: &Span, key: &str) -> Option<String> {
    attribute_of(&span.attributes, key)
}

/// The text form of the attribute `key` in `attributes`.
fn attribute_of(attributes: &[KeyValue], key: &str) -> Option<String> {
    let value = attributes
        .iter()
        .find(|attribute| attribute.key == key)?
        .value
        .as_ref()?
        .value
        .as_ref()?;
    Some(match value {
        Value::StringValue(text) => text.clone(),
        Value::IntValue(number) => number.to_string(),
        Value::BoolValue(flag) => flag.to_string(),
        other => format!("{other:?}"),
    })
}

/// The metric `name` of the last export that holds it.
fn last_metric(collector: &Collector, name: &str) -> Option<Metric> {
    collector
        .metrics()
        .into_iter()
        .flat_map(|request| request.resource_metrics)
        .flat_map(|metrics| metrics.scope_metrics)
        .flat_map(|scope| scope.metrics)
        .rfind(|metric| metric.name == name)
}

/// The value of the point of the sum or gauge `name`, in its last export, whose attributes
/// hold each pair of `attributes`.
fn last_value(collector: &Collector, name: &str, attributes: &[(&str, &str)]) -> Option<f64> {
    let points: Vec<NumberDataPoint> = match last_metric(collector, name)?.data? {
        Data::Sum(sum) => sum.data_points,
        Data::Gauge(gauge) => gauge.data_points,
        other => panic!("{name} is not a sum or a gauge: {other:?}"),
    };
    let point = points.into_iter().find(|point| {
        attributes
            .iter()
            .all(|(key, value)| attribute_of(&point.attributes, key).as_deref() == Some(value))
    })?;
    #[expect(clippy::cast_precision_loss, reason = "the counts of a test are small")]
    match point.value? {
        Number::AsInt(value) => Some(value as f64),
        Number::AsDouble(value) => Some(value),
    }
}

/// The value of `service.name` in `resource`.
fn service_name(resource: Option<&Resource>) -> Option<String> {
    resource?
        .attributes
        .iter()
        .find(|attribute| attribute.key == "service.name")
        .and_then(
            |attribute| match attribute.value.as_ref()?.value.as_ref()? {
                Value::StringValue(name) => Some(name.clone()),
                _ => None,
            },
        )
}

/// The service names of each export that `collector` got.
fn service_names(collector: &Collector) -> Vec<Option<String>> {
    let traces = collector.traces().into_iter().flat_map(|request| {
        request
            .resource_spans
            .into_iter()
            .map(|spans| service_name(spans.resource.as_ref()))
    });
    let metrics = collector.metrics().into_iter().flat_map(|request| {
        request
            .resource_metrics
            .into_iter()
            .map(|metrics| service_name(metrics.resource.as_ref()))
    });
    traces.chain(metrics).collect()
}

/// A server with `server.toml` set to `text`, run to its exit.
async fn refused_with_file(text: &str) -> Output {
    let config = ConfigDir::empty().file("server.toml", text);
    ServerBuilder::new()
        .env("MU_CONFIG", Some(config.path()))
        .output()
        .await
}

/// Set in the child test process of `helper_removes_the_otel_variables`.
const PROBE: &str = "MU_TEST_OTEL_PROBE";

/// Runs itself again as a child with the variable set, because `set_var` needs `unsafe`. The
/// child starts a process with the test environment, and checks that the process does not see
/// the variable.
#[test]
fn helper_removes_the_otel_variables() {
    if std::env::var_os(PROBE).is_some() {
        let mut command = Command::new("env");
        support::apply_test_env(&mut command);
        let output = command.output().expect("env runs");
        let seen = support::stdout(&output);
        assert!(output.status.success());
        assert!(!seen.contains("OTEL_EXPORTER_OTLP_COMPRESSION"), "{seen}");
        return;
    }
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "helper_removes_the_otel_variables",
            "--nocapture",
        ])
        .env(PROBE, "1")
        .env("OTEL_EXPORTER_OTLP_COMPRESSION", "gzip")
        .output()
        .expect("the test binary runs");
    assert!(output.status.success(), "{}", support::stdout(&output));
    assert!(support::stdout(&output).contains("1 passed"));
}

#[tokio::test]
async fn unknown_field_in_server_toml() {
    let output = refused_with_file("[telemetry]\nendpont = \"http://127.0.0.1:4318\"\n").await;
    fails_naming(&output, &["server.toml", "telemetry.endpont"]);
}

#[tokio::test]
async fn unknown_table_in_server_toml() {
    let output = refused_with_file("[logging]\nlevel = \"debug\"\n").await;
    fails_naming(&output, &["server.toml", "logging"]);
}

#[tokio::test]
async fn value_of_the_wrong_type() {
    let output = refused_with_file("[telemetry]\nendpoint = 4318\n").await;
    fails_naming(&output, &["server.toml", "telemetry.endpoint"]);
}

#[tokio::test]
async fn endpoint_that_is_not_a_url() {
    let output = refused_with_file("[telemetry]\nendpoint = \"not a url\"\n").await;
    fails_naming(&output, &["server.toml", "telemetry.endpoint"]);
}

#[tokio::test]
async fn endpoint_with_a_query() {
    let output =
        refused_with_file("[telemetry]\nendpoint = \"http://127.0.0.1:4318?tenant=a\"\n").await;
    fails_naming(&output, &["server.toml", "telemetry.endpoint"]);
}

#[tokio::test]
async fn grpc_endpoint_with_a_path() {
    let output = refused_with_file(
        "[telemetry]\nendpoint = \"http://127.0.0.1:4317/otel\"\nprotocol = \"grpc\"\n",
    )
    .await;
    fails_naming(&output, &["server.toml", "telemetry.endpoint"]);
}

#[cfg(unix)]
#[tokio::test]
async fn variable_that_is_not_utf8() {
    use std::os::unix::ffi::OsStringExt as _;

    let output = ServerBuilder::new()
        .env("MU_CONFIG", Some(support::empty_config_dir()))
        .env(ENDPOINT, Some(std::ffi::OsString::from_vec(vec![0xff])))
        .output()
        .await;
    fails_naming(&output, &[ENDPOINT, "UTF-8"]);
}

#[tokio::test]
async fn endpoint_with_another_scheme() {
    let output =
        refused_with_file("[telemetry]\nendpoint = \"ftp://collector.example:4318\"\n").await;
    fails_naming(&output, &["server.toml", "telemetry.endpoint", "ftp"]);
}

#[tokio::test]
async fn server_without_endpoint_writes_only_the_serving_line() {
    let run = Setup::new(&[]).one_send().await;
    assert!(run.status.success());
    assert_eq!(run.stderr.len(), 1, "{:?}", run.stderr);
    assert!(run.stderr[0].starts_with(SERVING), "{:?}", run.stderr);
}

#[tokio::test]
async fn traces_and_metrics_reach_the_collector() {
    let collector = Collector::start(Mode::Http).await;
    let run = Setup::endpoint(&collector.url()).one_send().await;
    assert!(run.status.success());
    for path in ["/v1/traces", "/v1/metrics"] {
        let requests: Vec<_> = collector
            .requests()
            .into_iter()
            .filter(|request| request.path == path)
            .collect();
        assert!(!requests.is_empty(), "{path}: {:?}", collector.paths());
        for request in requests {
            assert_eq!(request.method, "POST");
            assert_eq!(request.content_type, "application/x-protobuf");
        }
    }
    assert!(
        !run.stderr.iter().any(|line| line.contains(LATE_STOP)),
        "{:?}",
        run.stderr
    );
}

#[tokio::test]
async fn redirect_is_not_followed() {
    let collector = Collector::start(Mode::Http).await;
    Setup::new(&[])
        .env(
            TRACES_ENDPOINT,
            &format!("{}{}/traces", collector.url(), collector::REDIRECT),
        )
        .one_send()
        .await;
    let paths = collector.paths();
    assert!(
        paths
            .iter()
            .any(|path| path.starts_with(collector::REDIRECT)),
        "{paths:?}"
    );
    assert!(
        !paths.iter().any(|path| path == collector::ELSEWHERE),
        "{paths:?}"
    );
}

#[tokio::test]
async fn base_url_with_a_trailing_slash() {
    let collector = Collector::start(Mode::Http).await;
    Setup::endpoint(&format!("{}/", collector.url()))
        .one_send()
        .await;
    assert!(collector.got("/v1/traces"), "{:?}", collector.paths());
    assert!(collector.got("/v1/metrics"), "{:?}", collector.paths());
}

#[tokio::test]
async fn log_lines_stay() {
    let collector = Collector::start(Mode::Http).await;
    let run = Setup::endpoint(&collector.url()).run(&[]).await;
    assert!(
        run.stderr.iter().any(|line| line.starts_with(SERVING)),
        "{:?}",
        run.stderr
    );
}

#[tokio::test]
async fn variable_wins_over_the_file() {
    let a = Collector::start(Mode::Http).await;
    let b = Collector::start(Mode::Http).await;
    Setup::endpoint(&a.url())
        .env(ENDPOINT, &b.url())
        .one_send()
        .await;
    assert!(
        b.got("/v1/traces") && b.got("/v1/metrics"),
        "{:?}",
        b.paths()
    );
    assert!(a.requests().is_empty(), "{:?}", a.paths());
}

#[tokio::test]
async fn endpoint_of_the_traces() {
    let collector = Collector::start(Mode::Http).await;
    Setup::new(&[])
        .env(
            TRACES_ENDPOINT,
            &format!("{}/custom/traces", collector.url()),
        )
        .one_send()
        .await;
    assert!(collector.got("/custom/traces"), "{:?}", collector.paths());
    assert!(!collector.got("/v1/metrics"), "{:?}", collector.paths());
}

#[tokio::test]
async fn endpoint_of_the_metrics() {
    let collector = Collector::start(Mode::Http).await;
    Setup::new(&[])
        .env(
            METRICS_ENDPOINT,
            &format!("{}/custom/metrics", collector.url()),
        )
        .one_send()
        .await;
    assert!(collector.got("/custom/metrics"), "{:?}", collector.paths());
    assert!(!collector.got("/v1/traces"), "{:?}", collector.paths());
}

#[tokio::test]
async fn empty_variable_counts_as_unset() {
    let collector = Collector::start(Mode::Http).await;
    Setup::endpoint(&collector.url())
        .env(ENDPOINT, "")
        .one_send()
        .await;
    assert!(collector.got("/v1/traces"), "{:?}", collector.paths());
}

#[tokio::test]
async fn bad_variable_is_refused() {
    let output = Setup::new(&[])
        .env(ENDPOINT, "ftp://collector.example:4318")
        .output()
        .await;
    fails_naming(&output, &[ENDPOINT, "ftp"]);
}

#[tokio::test]
async fn bad_variable_of_one_signal_is_refused() {
    for name in [TRACES_ENDPOINT, METRICS_ENDPOINT] {
        for value in ["ftp://collector.example:4318/v1/x", "not a url"] {
            let output = Setup::new(&[]).env(name, value).output().await;
            fails_naming(&output, &[name]);
        }
    }
}

#[tokio::test]
async fn sdk_disabled_turns_the_export_off() {
    let collector = Collector::start(Mode::Http).await;
    let run = Setup::endpoint(&collector.url())
        .env(SDK_DISABLED, "TRUE")
        .one_send()
        .await;
    assert!(run.status.success());
    assert!(collector.requests().is_empty(), "{:?}", collector.paths());
}

#[tokio::test]
async fn sdk_disabled_with_another_value_is_named() {
    let collector = Collector::start(Mode::Http).await;
    let run = Setup::endpoint(&collector.url())
        .env(SDK_DISABLED, "1")
        .one_send()
        .await;
    assert!(
        run.stderr.iter().any(|line| line.contains(SDK_DISABLED)),
        "{:#?}",
        run.stderr
    );
    assert!(!collector.requests().is_empty());
}

#[tokio::test]
async fn sdk_disabled_with_a_bad_variable() {
    let run = Setup::new(&[])
        .env(SDK_DISABLED, "true")
        .env(ENDPOINT, "ftp://collector.example:4318")
        .run(&[])
        .await;
    assert!(run.status.success(), "{:?}", run.stderr);
}

#[tokio::test]
async fn sdk_disabled_with_an_unknown_field() {
    let output = Setup::new(&["endpont = \"http://127.0.0.1:4318\"".to_owned()])
        .env(SDK_DISABLED, "true")
        .output()
        .await;
    fails_naming(&output, &["server.toml", "telemetry.endpont"]);
}

#[tokio::test]
async fn default_service_name() {
    let collector = Collector::start(Mode::Http).await;
    Setup::endpoint(&collector.url()).run(&[]).await;
    let names = service_names(&collector);
    assert_ne!(names, Vec::<Option<String>>::new());
    assert!(
        names.iter().all(|name| name.as_deref() == Some("mu")),
        "{names:?}"
    );
}

#[tokio::test]
async fn empty_service_name() {
    let collector = Collector::start(Mode::Http).await;
    Setup::endpoint(&collector.url())
        .env(SERVICE_NAME, "")
        .run(&[])
        .await;
    let names = service_names(&collector);
    assert_ne!(names, Vec::<Option<String>>::new());
    assert!(
        names.iter().all(|name| name.as_deref() == Some("mu")),
        "{names:?}"
    );
}

#[tokio::test]
async fn service_name_from_the_variable() {
    let collector = Collector::start(Mode::Http).await;
    Setup::endpoint(&collector.url())
        .env(SERVICE_NAME, "mu-desk")
        .run(&[])
        .await;
    let names = service_names(&collector);
    assert_ne!(names, Vec::<Option<String>>::new());
    assert!(
        names.iter().all(|name| name.as_deref() == Some("mu-desk")),
        "{names:?}"
    );
}

#[tokio::test]
async fn grpc_from_the_file() {
    let collector = Collector::start(Mode::Grpc).await;
    Setup::new(&[
        format!("endpoint = \"{}\"", collector.url()),
        "protocol = \"grpc\"".to_owned(),
    ])
    .one_send()
    .await;
    assert!(collector.got(TRACE_EXPORT), "{:?}", collector.paths());
    assert!(collector.got(METRICS_EXPORT), "{:?}", collector.paths());
}

#[tokio::test]
async fn variable_wins_over_the_protocol_of_the_file() {
    let collector = Collector::start(Mode::Grpc).await;
    Setup::new(&[
        format!("endpoint = \"{}\"", collector.url()),
        "protocol = \"http/protobuf\"".to_owned(),
    ])
    .env(PROTOCOL, "grpc")
    .one_send()
    .await;
    assert!(collector.got(TRACE_EXPORT), "{:?}", collector.paths());
}

#[tokio::test]
async fn protocol_of_one_signal() {
    let http = Collector::start(Mode::Http).await;
    let grpc = Collector::start(Mode::Grpc).await;
    Setup::new(&[])
        .env(TRACES_ENDPOINT, &format!("{}/custom/traces", http.url()))
        .env(METRICS_ENDPOINT, &grpc.url())
        .env(METRICS_PROTOCOL, "grpc")
        .one_send()
        .await;
    assert!(http.got("/custom/traces"), "{:?}", http.paths());
    assert_ne!(http.traces(), Vec::new());
    assert!(grpc.got(METRICS_EXPORT), "{:?}", grpc.paths());
}

#[tokio::test]
async fn http_json_is_refused() {
    let collector = Collector::start(Mode::Http).await;
    let output = Setup::endpoint(&collector.url())
        .env(PROTOCOL, "http/json")
        .output()
        .await;
    fails_naming(&output, &[PROTOCOL]);
    assert!(
        !support::stderr(&output).contains("http/json"),
        "{}",
        support::stderr(&output)
    );
}

#[tokio::test]
async fn unknown_protocol_in_the_file() {
    let output = Setup::new(&[
        "endpoint = \"http://127.0.0.1:4318\"".to_owned(),
        "protocol = \"thrift\"".to_owned(),
    ])
    .output()
    .await;
    fails_naming(&output, &["server.toml", "telemetry.protocol"]);
}

#[tokio::test]
async fn unknown_protocol_with_the_export_off() {
    let run = Setup::new(&[]).env(PROTOCOL, "http/json").run(&[]).await;
    assert!(run.status.success(), "{:?}", run.stderr);
}

#[tokio::test]
async fn relative_certificate_path() {
    let collector = Collector::start(Mode::Https).await;
    let authority = std::fs::read_to_string(collector.authority()).unwrap();
    Setup::new(&[
        format!("endpoint = \"{}\"", collector.url()),
        "certificate = \"certs/ca.pem\"".to_owned(),
    ])
    .file("certs/ca.pem", &authority)
    .one_send()
    .await;
    assert!(collector.got("/v1/traces"), "{:?}", collector.paths());
}

#[tokio::test]
async fn https_with_a_certificate_file() {
    let collector = Collector::start(Mode::Https).await;
    Setup::new(&[
        format!("endpoint = \"{}\"", collector.url()),
        format!("certificate = \"{}\"", collector.authority().display()),
    ])
    .one_send()
    .await;
    assert!(collector.got("/v1/traces"), "{:?}", collector.paths());
    assert!(collector.got("/v1/metrics"), "{:?}", collector.paths());
}

#[tokio::test]
async fn grpc_with_tls() {
    let collector = Collector::start(Mode::GrpcTls).await;
    Setup::new(&[
        format!("endpoint = \"{}\"", collector.url()),
        "protocol = \"grpc\"".to_owned(),
    ])
    .env(CERTIFICATE, collector.authority().to_str().unwrap())
    .one_send()
    .await;
    assert!(collector.got(TRACE_EXPORT), "{:?}", collector.paths());
}

#[tokio::test]
async fn grpc_with_tls_and_an_upper_case_scheme() {
    let collector = Collector::start(Mode::GrpcTls).await;
    let url = collector.url().replacen("https", "HTTPS", 1);
    Setup::new(&[
        format!("endpoint = \"{url}\""),
        "protocol = \"grpc\"".to_owned(),
    ])
    .env(CERTIFICATE, collector.authority().to_str().unwrap())
    .one_send()
    .await;
    assert!(collector.got(TRACE_EXPORT), "{:?}", collector.paths());
}

#[tokio::test]
async fn silent_traces_collector_does_not_stop_the_metrics() {
    let silent = Collector::start(Mode::Silent).await;
    let collector = Collector::start(Mode::Http).await;
    Setup::new(&[])
        .env(TRACES_ENDPOINT, &format!("{}/v1/traces", silent.url()))
        .env(METRICS_ENDPOINT, &format!("{}/v1/metrics", collector.url()))
        .one_send()
        .await;
    assert!(collector.got("/v1/metrics"), "{:?}", collector.paths());
}

#[tokio::test]
async fn collector_that_the_server_does_not_trust() {
    let collector = Collector::start(Mode::Https).await;
    let run = Setup::endpoint(&collector.url()).one_send().await;
    assert!(run.status.success(), "{:?}", run.stderr);
    assert!(collector.requests().is_empty(), "{:?}", collector.paths());
    assert!(
        run.stderr.iter().any(|line| line.contains("ERROR")
            && (line.contains("Processor.ExportError") || line.contains("Reader.ExportError"))),
        "{:#?}",
        run.stderr
    );
    assert!(
        run.stderr
            .iter()
            .any(|line| line.contains("WARN") && line.contains("Export.Failed.Exhausted")),
        "{:#?}",
        run.stderr
    );
}

#[tokio::test]
async fn collector_on_a_closed_port() {
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let run = Setup::endpoint(&format!("http://127.0.0.1:{port}"))
        .one_send()
        .await;
    assert!(run.status.success(), "{:?}", run.stderr);
    assert!(
        run.stderr
            .iter()
            .any(|line| line.contains("WARN") && line.contains(FAILED_STOP)),
        "{:#?}",
        run.stderr
    );
    assert!(
        !run.stderr.iter().any(|line| line.contains(LATE_STOP)),
        "{:#?}",
        run.stderr
    );
}

#[tokio::test]
async fn missing_certificate_file() {
    let output = Setup::new(&[
        "endpoint = \"http://127.0.0.1:4318\"".to_owned(),
        "certificate = \"/nonexistent/ca.pem\"".to_owned(),
    ])
    .output()
    .await;
    fails_naming(
        &output,
        &[
            "server.toml",
            "telemetry.certificate",
            "/nonexistent/ca.pem",
        ],
    );
}

#[tokio::test]
async fn file_with_no_certificate() {
    let collector = Collector::start(Mode::Http).await;
    let bad = collector.dir().join("bad.pem");
    std::fs::write(&bad, "no certificate here\n").unwrap();
    let output = Setup::endpoint(&collector.url())
        .env(CERTIFICATE, bad.to_str().unwrap())
        .output()
        .await;
    fails_naming(&output, &[CERTIFICATE]);
}

#[tokio::test]
async fn setting_that_the_sdk_ignores_is_written_to_stderr() {
    let collector = Collector::start(Mode::Http).await;
    let variable = "OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE";
    let run = Setup::endpoint(&collector.url())
        .env(variable, "deltaa")
        .one_send()
        .await;
    assert!(
        run.stderr.iter().any(|line| line.contains(variable)),
        "{:#?}",
        run.stderr
    );
}

#[tokio::test]
async fn configuration_directory_that_is_a_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("not-a-directory");
    std::fs::write(&file, "").unwrap();
    let output = ServerBuilder::new()
        .env("MU_CONFIG", Some(&file))
        .fake_worlds(&["alpha/aria"])
        .output()
        .await;
    fails_naming(&output, &[file.to_str().unwrap()]);
}

#[tokio::test]
async fn certificate_that_does_not_parse() {
    for protocol in ["http/protobuf", "grpc"] {
        let dir = tempfile::tempdir().unwrap();
        let bad = dir.path().join("bad.pem");
        std::fs::write(
            &bad,
            "-----BEGIN CERTIFICATE-----\nAAECAwQF\n-----END CERTIFICATE-----\n",
        )
        .unwrap();
        let output = Setup::endpoint("https://127.0.0.1:4318")
            .env("OTEL_EXPORTER_OTLP_PROTOCOL", protocol)
            .env(CERTIFICATE, bad.to_str().unwrap())
            .output()
            .await;
        fails_naming(&output, &[CERTIFICATE, bad.to_str().unwrap()]);
    }
}

#[tokio::test]
async fn span_of_a_send() {
    let collector = Collector::start(Mode::Http).await;
    Setup::endpoint(&collector.url()).one_send().await;
    let span = one_span(&collector, "mu.v1.PortalService/Send");
    assert_eq!(attribute(&span, "mu.world").as_deref(), Some("alpha/aria"));
    assert_eq!(
        attribute(&span, "rpc.system").as_deref(),
        Some("connect_rpc")
    );
    assert_eq!(
        attribute(&span, "rpc.service").as_deref(),
        Some("mu.v1.PortalService")
    );
    assert_eq!(attribute(&span, "rpc.method").as_deref(), Some("Send"));
}

#[tokio::test]
async fn span_of_a_subscribe() {
    let collector = Collector::start(Mode::Http).await;
    let mut session = Setup::endpoint(&collector.url()).start().await;
    session.client.send("alpha/aria", "hello").await.unwrap();
    session.line("alpha/aria", "You say, \"hello\"").await;
    stop(session.server).await;
    let subscribe = one_span(&collector, "mu.v1.PortalService/Subscribe");
    let send = one_span(&collector, "mu.v1.PortalService/Send");
    assert!(
        subscribe.end_time_unix_nano <= send.start_time_unix_nano,
        "Subscribe ends at {}, Send starts at {}",
        subscribe.end_time_unix_nano,
        send.start_time_unix_nano
    );
}

#[tokio::test]
async fn span_of_a_failed_call() {
    let collector = Collector::start(Mode::Http).await;
    let session = Setup::endpoint(&collector.url()).start().await;
    session.client.connect("zeta/zed").await.unwrap_err();
    stop(session.server).await;
    let span = one_span(&collector, "mu.v1.PortalService/Connect");
    let status = span.status.as_ref().expect("the span has a status");
    assert_eq!(status.code, StatusCode::Error as i32);
    assert_eq!(
        attribute(&span, "rpc.connect_rpc.error_code").as_deref(),
        Some("not_found")
    );
}

#[tokio::test]
async fn subscribe_past_the_end() {
    let collector = Collector::start(Mode::Http).await;
    let session = Setup::endpoint(&collector.url()).start().await;
    let mut stream = session.client.subscribe(1_000_000).await.unwrap();
    while stream.next().await.is_ok_and(|item| item.is_some()) {}
    stop(session.server).await;
    let failed: Vec<_> = spans_named(&collector, "mu.v1.PortalService/Subscribe")
        .into_iter()
        .filter(|span| {
            span.status
                .as_ref()
                .is_some_and(|status| status.code == StatusCode::Error as i32)
        })
        .collect();
    assert_eq!(failed.len(), 1, "{failed:?}");
}

#[tokio::test]
async fn command_under_its_call() {
    let collector = Collector::start(Mode::Http).await;
    Setup::endpoint(&collector.url()).one_send().await;
    let call = one_span(&collector, "mu.v1.PortalService/Send");
    let commands: Vec<_> = spans_named(&collector, "command")
        .into_iter()
        .filter(|span| attribute(span, "mu.command").as_deref() == Some("send"))
        .collect();
    assert_eq!(commands.len(), 1, "{commands:?}");
    let command = &commands[0];
    assert_eq!(
        attribute(command, "mu.world").as_deref(),
        Some("alpha/aria")
    );
    assert_eq!(attribute(command, "mu.origin").as_deref(), Some("player"));
    assert_eq!(command.trace_id, call.trace_id);
    assert_eq!(command.parent_span_id, call.span_id);
}

#[tokio::test]
async fn command_of_the_server() {
    let collector = Collector::start(Mode::Http).await;
    Setup::endpoint(&collector.url()).run(&[]).await;
    let connects: Vec<_> = spans_named(&collector, "command")
        .into_iter()
        .filter(|span| attribute(span, "mu.command").as_deref() == Some("connect"))
        .collect();
    assert_eq!(connects.len(), 1, "{connects:?}");
    assert_eq!(
        attribute(&connects[0], "mu.origin").as_deref(),
        Some("system")
    );
}

#[tokio::test]
async fn connection_of_a_fake_world() {
    let collector = Collector::start(Mode::Http).await;
    Setup::endpoint(&collector.url()).run(&[]).await;
    let span = one_span(&collector, "world.connection");
    assert_eq!(attribute(&span, "mu.world").as_deref(), Some("alpha/aria"));
}

#[tokio::test]
async fn events_of_a_world() {
    let collector = Collector::start(Mode::Http).await;
    Setup::endpoint(&collector.url()).run(&["one", "two"]).await;
    let inputs = last_value(
        &collector,
        "mu.journal.events",
        &[("mu.world", "alpha/aria"), ("mu.event.kind", "input")],
    );
    assert_eq!(inputs, Some(2.0));
}

#[tokio::test]
async fn lag_of_a_subscriber() {
    let collector = Collector::start(Mode::Http).await;
    let session = Setup::endpoint(&collector.url()).start().await;
    assert!(session.seen().len() >= 3, "{:?}", session.seen());
    stop(session.server).await;
    let metric = last_metric(&collector, "mu.subscriber.lag").expect("the lag is exported");
    let Some(Data::Histogram(histogram)) = metric.data else {
        panic!("not a histogram: {metric:?}");
    };
    let count: u64 = histogram.data_points.iter().map(|point| point.count).sum();
    assert!(count >= 3, "{count}");
}

#[tokio::test]
async fn world_that_never_connects() {
    let collector = Collector::start(Mode::Http).await;
    let config = ConfigDir::base().file(
        "server.toml",
        &format!("[telemetry]\nendpoint = \"{}\"\n", collector.url()),
    );
    let server = ServerBuilder::new()
        .env("MU_CONFIG", Some(config.path()))
        .start()
        .await;
    stop(server).await;
    let state = |name| {
        last_value(
            &collector,
            "mu.world.state",
            &[("mu.world", "alpha/aria"), ("mu.world.state.name", name)],
        )
    };
    assert_eq!(state("disconnected"), Some(1.0));
}

#[tokio::test]
async fn state_at_the_stop() {
    let collector = Collector::start(Mode::Http).await;
    Setup::endpoint(&collector.url()).run(&[]).await;
    let state = |name| {
        last_value(
            &collector,
            "mu.world.state",
            &[("mu.world", "alpha/aria"), ("mu.world.state.name", name)],
        )
    };
    assert_eq!(state("disconnected"), Some(1.0));
    assert_eq!(state("connected"), Some(0.0));
}

/// Whether `text` occurs in a body that `collector` got.
fn in_a_body(collector: &Collector, text: &str) -> bool {
    collector.requests().iter().any(|request| {
        request
            .body
            .windows(text.len())
            .any(|window| window == text.as_bytes())
    })
}

#[tokio::test]
async fn password_stays_out() {
    let collector = Collector::start(Mode::Http).await;
    let config = ConfigDir::empty().file(
        "server.toml",
        &format!("[telemetry]\nendpoint = \"{}\"\n", collector.url()),
    );
    let builder = ServerBuilder::new()
        .env("MU_CONFIG", Some(config.path()))
        .fake_world("alpha/aria", "ALPHA_PW", PW);
    let mut session = Session::start(builder).await;
    session.welcome("alpha/aria").await;
    let text = format!("say {PW}");
    session.client.send("alpha/aria", &text).await.unwrap();
    let run = stop(session.server).await;
    assert!(!collector.requests().is_empty());
    assert!(!in_a_body(&collector, PW), "a body holds the password");
    assert!(
        run.stderr.iter().all(|line| !line.contains(PW)),
        "{:?}",
        run.stderr
    );
}

#[tokio::test]
async fn line_text_stays_out() {
    let collector = Collector::start(Mode::Http).await;
    Setup::endpoint(&collector.url())
        .run(&["say marker-7f3a"])
        .await;
    assert!(!collector.requests().is_empty());
    assert!(
        !in_a_body(&collector, "marker-7f3a"),
        "a body holds the line"
    );
}

/// The number of calls that make more than twice the 2048 spans of the default queue of the
/// batch span processor. Each call makes two spans: `rpc` and `command`.
const CALLS: usize = 2100;

/// The longest time from SIGTERM to the exit: the bound of the stop and a margin.
const EXIT_BOUND: std::time::Duration = std::time::Duration::from_secs(7);

async fn silent_collector_with(protocol: &str) {
    let collector = Collector::start(Mode::Silent).await;
    let session = Setup::new(&[
        format!("endpoint = \"{}\"", collector.url()),
        format!("protocol = \"{protocol}\""),
    ])
    .start()
    .await;
    for n in 0..CALLS {
        let seq = session
            .client
            .send("alpha/aria", &format!("line {n}"))
            .await;
        assert!(matches!(seq, Ok(Some(_))), "call {n}: {seq:?}");
    }
    let mut server = session.server;
    let start = std::time::Instant::now();
    server.signal(Signal::TERM);
    let status = server.wait_exit().await;
    let elapsed = start.elapsed();
    assert!(status.success(), "{status:?}");
    assert!(
        elapsed <= EXIT_BOUND,
        "{protocol}: the exit took {elapsed:?}"
    );
    server.wait_stderr(|_| false).await;
    let stderr = server.stderr();
    assert!(
        stderr
            .iter()
            .any(|line| line.contains("WARN") && line.contains(LATE_STOP)),
        "{protocol}: {stderr:?}"
    );
}

#[tokio::test]
async fn silent_collector_does_not_delay_a_send() {
    for protocol in ["http/protobuf", "grpc"] {
        silent_collector_with(protocol).await;
    }
}
