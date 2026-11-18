use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;

use proptest::prelude::*;

use super::*;

const BASE: &str = "http://h:4318";

fn table(endpoint: Option<&str>) -> TelemetryTable {
    TelemetryTable {
        endpoint: endpoint.map(str::to_owned),
        ..TelemetryTable::default()
    }
}

/// The table with `protocol` and `endpoint`.
fn protocol_table(protocol: &str, endpoint: &str) -> TelemetryTable {
    TelemetryTable {
        protocol: Some(protocol.to_owned()),
        ..table(Some(endpoint))
    }
}

fn run(
    vars: &[(&str, &str)],
    file: Option<&TelemetryTable>,
) -> Result<Option<Export>, SettingError> {
    let vars: HashMap<String, String> = vars
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect();
    export(
        |name| vars.get(name).map(OsString::from),
        file.unwrap_or(&TelemetryTable::default()),
    )
}

pub(super) fn on(vars: &[(&str, &str)], file: Option<&TelemetryTable>) -> Export {
    run(vars, file).unwrap().expect("the export is on")
}

fn endpoints(export: &Export) -> (Option<&str>, Option<&str>) {
    (
        export
            .traces
            .as_ref()
            .map(|signal| signal.endpoint.as_str()),
        export
            .metrics
            .as_ref()
            .map(|signal| signal.endpoint.as_str()),
    )
}

fn refused_by(vars: &[(&str, &str)], file: Option<&TelemetryTable>) -> SettingSource {
    run(vars, file).unwrap_err().place()
}

fn pem_file(text: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ca.pem");
    std::fs::write(&path, text).unwrap();
    (dir, path)
}

fn authority_pem() -> String {
    let key = rcgen::KeyPair::generate().unwrap();
    let mut params = rcgen::CertificateParams::new(Vec::new()).unwrap();
    params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    params.self_signed(&key).unwrap().pem()
}

#[test]
fn no_endpoint_means_no_export() {
    assert_eq!(run(&[], None), Ok(None));
    assert_eq!(run(&[], Some(&TelemetryTable::default())), Ok(None));
}

#[test]
fn file_endpoint_gets_the_paths() {
    let export = on(&[], Some(&table(Some(BASE))));
    assert_eq!(
        endpoints(&export),
        (
            Some("http://h:4318/v1/traces"),
            Some("http://h:4318/v1/metrics")
        )
    );
}

#[test]
fn base_variable_wins_over_the_file() {
    let export = on(&[(ENDPOINT, "http://b:1")], Some(&table(Some(BASE))));
    assert_eq!(
        endpoints(&export),
        (Some("http://b:1/v1/traces"), Some("http://b:1/v1/metrics"))
    );
}

#[test]
fn signal_variable_wins_and_gets_no_path() {
    let export = on(
        &[(TRACES_ENDPOINT, "http://t:1/x"), (ENDPOINT, "http://b:1")],
        Some(&table(Some(BASE))),
    );
    assert_eq!(
        endpoints(&export),
        (Some("http://t:1/x"), Some("http://b:1/v1/metrics"))
    );
}

#[test]
fn one_signal_without_an_endpoint() {
    let export = on(&[(METRICS_ENDPOINT, "http://m:1/x")], None);
    assert_eq!(endpoints(&export), (None, Some("http://m:1/x")));
}

#[test]
fn empty_variable_counts_as_unset() {
    let export = on(
        &[(ENDPOINT, ""), (TRACES_ENDPOINT, ""), (SERVICE_NAME, "")],
        Some(&table(Some(BASE))),
    );
    assert_eq!(
        endpoints(&export),
        (
            Some("http://h:4318/v1/traces"),
            Some("http://h:4318/v1/metrics")
        )
    );
}

#[test]
fn sdk_disabled_true_in_any_case_turns_the_export_off() {
    for value in ["true", "TrUe"] {
        assert_eq!(
            run(&[(SDK_DISABLED, value)], Some(&table(Some(BASE)))),
            Ok(None)
        );
    }
}

#[test]
fn sdk_disabled_with_another_value_keeps_the_order() {
    for value in ["false", "1", ""] {
        assert!(
            on(&[(SDK_DISABLED, value)], Some(&table(Some(BASE))))
                .traces
                .is_some()
        );
    }
}

#[test]
fn sdk_disabled_value_that_is_ignored() {
    let ignored = |value: &str| sdk_disabled_is_ignored(|_| Some(OsString::from(value)));
    for value in ["1", "yes", "on", " true"] {
        assert!(ignored(value), "{value}");
    }
    for value in ["", "true", "TRUE", "false", "False"] {
        assert!(!ignored(value), "{value}");
    }
    assert!(!sdk_disabled_is_ignored(|_| None));
}

#[test]
fn sdk_disabled_reads_nothing_else() {
    let file = TelemetryTable {
        endpoint: Some("not a url".to_owned()),
        protocol: Some("thrift".to_owned()),
        certificate: Some(PathBuf::from("/nonexistent/ca.pem")),
    };
    let vars = [
        (SDK_DISABLED, "true"),
        (ENDPOINT, "ftp://collector.example:4318"),
        (TRACES_ENDPOINT, "not a url"),
        (PROTOCOL, "http/json"),
        (CERTIFICATE, "/nonexistent/ca.pem"),
    ];
    assert_eq!(run(&vars, Some(&file)), Ok(None));
}

#[cfg(unix)]
#[test]
fn sdk_disabled_reads_no_variable_that_is_not_utf8() {
    use std::os::unix::ffi::OsStringExt as _;

    let result = export(
        |asked| match asked {
            SDK_DISABLED => Some("true".into()),
            ENDPOINT | SERVICE_NAME => Some(OsString::from_vec(vec![0xff])),
            _ => None,
        },
        &TelemetryTable::default(),
    );
    assert_eq!(result, Ok(None));
}

#[test]
fn http_protobuf_is_the_default() {
    let export = on(&[], Some(&table(Some(BASE))));
    assert_eq!(export.traces.unwrap().protocol, Protocol::HttpProtobuf);
}

#[test]
fn grpc_from_each_source() {
    let file = protocol_table("grpc", BASE);
    let from_file = on(&[], Some(&file));
    assert_eq!(from_file.traces.unwrap().protocol, Protocol::Grpc);
    for name in [PROTOCOL, TRACES_PROTOCOL] {
        let export = on(&[(name, "grpc")], Some(&table(Some(BASE))));
        assert_eq!(export.traces.unwrap().protocol, Protocol::Grpc, "{name}");
    }
    let export = on(&[(METRICS_PROTOCOL, "grpc")], Some(&table(Some(BASE))));
    assert_eq!(export.metrics.unwrap().protocol, Protocol::Grpc);
    assert_eq!(export.traces.unwrap().protocol, Protocol::HttpProtobuf);
}

#[test]
fn order_of_the_protocol() {
    let file = protocol_table("grpc", BASE);
    let export = on(&[(PROTOCOL, "http/protobuf")], Some(&file));
    assert_eq!(export.traces.unwrap().protocol, Protocol::HttpProtobuf);
    let export = on(
        &[(PROTOCOL, "grpc"), (TRACES_PROTOCOL, "http/protobuf")],
        Some(&file),
    );
    assert_eq!(export.traces.unwrap().protocol, Protocol::HttpProtobuf);
    assert_eq!(export.metrics.unwrap().protocol, Protocol::Grpc);
}

#[test]
fn http_json_with_an_endpoint_is_refused_and_named() {
    for name in [PROTOCOL, TRACES_PROTOCOL, METRICS_PROTOCOL] {
        assert_eq!(
            refused_by(&[(name, "http/json")], Some(&table(Some(BASE)))),
            SettingSource::Variable(name)
        );
    }
    let file = protocol_table("http/json", BASE);
    assert_eq!(
        refused_by(&[], Some(&file)),
        SettingSource::Field(TableField::Protocol)
    );
}

#[test]
fn bad_protocol_is_not_echoed() {
    let value = "https://u:hunter2@collector.example";
    for vars in [&[(PROTOCOL, value)][..], &[(TRACES_PROTOCOL, value)][..]] {
        let message = run(vars, Some(&table(Some(BASE)))).unwrap_err().to_string();
        assert!(!message.contains("hunter2"), "{message}");
    }
}

#[test]
fn http_json_without_an_export_is_accepted() {
    assert_eq!(run(&[(PROTOCOL, "http/json")], None), Ok(None));
    assert_eq!(
        run(
            &[(PROTOCOL, "http/json"), (SDK_DISABLED, "true")],
            Some(&table(Some(BASE)))
        ),
        Ok(None)
    );
}

#[test]
fn empty_protocol_variable_counts_as_unset() {
    let file = protocol_table("grpc", BASE);
    let export = on(&[(TRACES_PROTOCOL, ""), (PROTOCOL, "")], Some(&file));
    assert_eq!(export.traces.unwrap().protocol, Protocol::Grpc);
}

#[test]
fn bad_endpoint_variables_are_refused_and_named() {
    for name in [ENDPOINT, TRACES_ENDPOINT, METRICS_ENDPOINT] {
        for value in [
            "ftp://collector.example:4318",
            "not a url",
            "collector.example:4318",
            "http:///v1",
        ] {
            assert_eq!(
                refused_by(&[(name, value)], None),
                SettingSource::Variable(name),
                "{name}={value}"
            );
        }
    }
}

#[test]
fn bad_file_endpoint_is_refused_and_named() {
    for value in ["not a url", "ftp://c:1", "http://127.0.0.1:4318?tenant=a"] {
        assert_eq!(
            refused_by(&[], Some(&table(Some(value)))),
            SettingSource::Field(TableField::Endpoint)
        );
    }
}

#[test]
fn bad_file_endpoint_under_a_set_variable_is_refused() {
    let vars = [(ENDPOINT, "http://127.0.0.1:4318")];
    assert_eq!(
        refused_by(&vars, Some(&table(Some("not a url")))),
        SettingSource::Field(TableField::Endpoint)
    );
}

#[test]
fn scheme_is_named() {
    let message = run(&[(ENDPOINT, "ftp://collector.example:4318")], None)
        .unwrap_err()
        .to_string();
    assert!(message.contains("ftp"), "{message}");
}

#[test]
fn query_or_fragment_in_the_base_is_refused() {
    for value in ["http://h:1?tenant=a", "http://h:1#top"] {
        assert_eq!(
            refused_by(&[(ENDPOINT, value)], None),
            SettingSource::Variable(ENDPOINT)
        );
    }
}

#[test]
fn fragment_in_a_signal_variable_is_refused() {
    for name in [TRACES_ENDPOINT, METRICS_ENDPOINT] {
        assert_eq!(
            refused_by(&[(name, "http://t:1/x#top")], None),
            SettingSource::Variable(name)
        );
    }
}

#[test]
fn query_in_a_signal_variable_is_used_as_it_is() {
    let export = on(&[(TRACES_ENDPOINT, "http://t:1/x?tenant=a")], None);
    assert_eq!(endpoints(&export).0, Some("http://t:1/x?tenant=a"));
}

#[test]
fn join_of_a_base_with_a_slash_and_with_a_path() {
    assert_eq!(
        join("http://h:4318/", SignalKind::Traces, Protocol::HttpProtobuf),
        "http://h:4318/v1/traces"
    );
    assert_eq!(
        join(
            "http://h:4318/",
            SignalKind::Metrics,
            Protocol::HttpProtobuf
        ),
        "http://h:4318/v1/metrics"
    );
    assert_eq!(
        join(
            "http://h:4318/otel",
            SignalKind::Traces,
            Protocol::HttpProtobuf
        ),
        "http://h:4318/otel/v1/traces"
    );
    assert_eq!(
        join(
            "http://h:4318/otel",
            SignalKind::Metrics,
            Protocol::HttpProtobuf
        ),
        "http://h:4318/otel/v1/metrics"
    );
}

#[test]
fn grpc_base_gets_no_path() {
    assert_eq!(
        join("http://h:4317", SignalKind::Traces, Protocol::Grpc),
        "http://h:4317"
    );
    let export = on(&[(PROTOCOL, "grpc")], Some(&table(Some(BASE))));
    assert_eq!(endpoints(&export), (Some(BASE), Some(BASE)));
}

#[test]
fn grpc_endpoint_with_a_path_or_a_query_is_refused_and_named() {
    let grpc = protocol_table("grpc", "http://h:4317/otel");
    assert_eq!(
        refused_by(&[], Some(&grpc)),
        SettingSource::Field(TableField::Endpoint)
    );
    assert_eq!(
        refused_by(
            &[(PROTOCOL, "grpc"), (ENDPOINT, "http://h:4317/otel")],
            None
        ),
        SettingSource::Variable(ENDPOINT)
    );
    for value in ["http://h:4317/v1/traces", "http://h:4317?tenant=a"] {
        assert_eq!(
            refused_by(&[(PROTOCOL, "grpc"), (TRACES_ENDPOINT, value)], None),
            SettingSource::Variable(TRACES_ENDPOINT),
            "{value}"
        );
    }
}

#[test]
fn grpc_source_with_a_path_that_loses_is_accepted() {
    let grpc = protocol_table("grpc", "http://h:4317/otel");
    let export = on(&[(ENDPOINT, "http://h:4317")], Some(&grpc));
    assert_eq!(
        endpoints(&export),
        (Some("http://h:4317"), Some("http://h:4317"))
    );
    let export = on(
        &[
            (PROTOCOL, "grpc"),
            (ENDPOINT, "http://h:4317/otel"),
            (TRACES_ENDPOINT, "http://t:4317"),
            (METRICS_ENDPOINT, "http://m:4317"),
        ],
        None,
    );
    assert_eq!(
        endpoints(&export),
        (Some("http://t:4317"), Some("http://m:4317"))
    );
}

#[test]
fn grpc_endpoint_with_a_slash_is_accepted() {
    let export = on(&[(PROTOCOL, "grpc"), (ENDPOINT, "http://h:4317/")], None);
    assert_eq!(
        endpoints(&export),
        (Some("http://h:4317/"), Some("http://h:4317/"))
    );
}

#[test]
fn https_is_found_in_any_case() {
    let export = on(&[(ENDPOINT, "HTTPS://h:4318")], None);
    assert!(export.traces.unwrap().is_https());
    let export = on(&[(ENDPOINT, "http://h:4318")], None);
    assert!(!export.traces.unwrap().is_https());
}

#[test]
fn debug_hides_the_endpoint() {
    let export = on(&[(ENDPOINT, "https://u:hunter2@h:4318")], None);
    let shown = format!("{export:?}");
    assert!(!shown.contains("hunter2"), "{shown}");
}

#[cfg(unix)]
#[test]
fn variable_that_is_not_utf8_is_refused_and_named() {
    use std::os::unix::ffi::OsStringExt as _;

    for name in [
        SDK_DISABLED,
        ENDPOINT,
        TRACES_ENDPOINT,
        PROTOCOL,
        SERVICE_NAME,
    ] {
        let bad = |asked: &str| (asked == name).then(|| OsString::from_vec(vec![0xff]));
        let result = export(
            |asked| bad(asked).or_else(|| (asked == METRICS_ENDPOINT).then(|| "http://m:1".into())),
            &TelemetryTable::default(),
        );
        assert_eq!(
            result.unwrap_err().place(),
            SettingSource::Variable(name),
            "{name}"
        );
    }
}

#[cfg(unix)]
#[test]
fn certificate_variable_that_is_not_utf8_is_a_path() {
    use std::os::unix::ffi::OsStringExt as _;

    let path = OsString::from_vec(b"/nonexistent/\xff.pem".to_vec());
    let result = export(
        |asked| match asked {
            ENDPOINT => Some("http://h:1".into()),
            CERTIFICATE => Some(path.clone()),
            _ => None,
        },
        &TelemetryTable::default(),
    );
    let error = result.unwrap_err();
    assert_eq!(error.place(), SettingSource::Variable(CERTIFICATE));
    assert!(!error.to_string().contains("UTF-8"), "{error}");
}

#[test]
fn certificate_variable_wins_over_the_field() {
    let (_dir, good) = pem_file(&authority_pem());
    let file = TelemetryTable {
        certificate: Some(PathBuf::from("/nonexistent/ca.pem")),
        ..table(Some(BASE))
    };
    let export = on(&[(CERTIFICATE, good.to_str().unwrap())], Some(&file));
    assert_eq!(export.authorities.certificates().len(), 1);
    assert_eq!(
        refused_by(&[], Some(&file)),
        SettingSource::Field(TableField::Certificate)
    );
}

#[test]
fn missing_certificate_file_names_the_path() {
    let message = run(
        &[(CERTIFICATE, "/nonexistent/ca.pem")],
        Some(&table(Some(BASE))),
    )
    .unwrap_err()
    .to_string();
    assert!(message.contains(CERTIFICATE), "{message}");
    assert!(message.contains("/nonexistent/ca.pem"), "{message}");
}

#[test]
fn file_with_no_certificate_is_refused() {
    let (_dir, path) = pem_file("no certificate here\n");
    assert_eq!(
        refused_by(
            &[(CERTIFICATE, path.to_str().unwrap())],
            Some(&table(Some(BASE)))
        ),
        SettingSource::Variable(CERTIFICATE)
    );
}

/// A PEM file whose `CERTIFICATE` section holds bytes that are not a certificate.
const BAD_DER_PEM: &str = "-----BEGIN CERTIFICATE-----\nAAECAwQF\n-----END CERTIFICATE-----\n";

#[test]
fn certificate_that_does_not_parse_is_refused_and_named() {
    let (_dir, path) = pem_file(BAD_DER_PEM);
    let message = run(
        &[(CERTIFICATE, path.to_str().unwrap())],
        Some(&table(Some(BASE))),
    )
    .unwrap_err()
    .to_string();
    assert!(message.contains(CERTIFICATE), "{message}");
    assert!(message.contains(path.to_str().unwrap()), "{message}");
    assert!(message.contains("certificate 1 is not valid"), "{message}");
}

#[test]
fn debug_hides_the_certificate_file() {
    let (_dir, path) = pem_file(&authority_pem());
    let export = on(
        &[(CERTIFICATE, path.to_str().unwrap())],
        Some(&table(Some(BASE))),
    );
    let shown = format!("{:?}", export.authorities);
    assert!(!shown.contains("BEGIN"), "{shown}");
    assert!(shown.contains("certificates: 1"), "{shown}");
}

#[test]
fn no_certificate_means_the_system_roots_only() {
    assert_eq!(
        on(&[], Some(&table(Some(BASE)))).authorities.certificates(),
        &[]
    );
}

#[test]
fn service_name_unset_empty_and_set() {
    let file = table(Some(BASE));
    assert_eq!(on(&[], Some(&file)).service_name, "mu");
    assert_eq!(on(&[(SERVICE_NAME, "")], Some(&file)).service_name, "mu");
    assert_eq!(
        on(&[(SERVICE_NAME, "mu-desk")], Some(&file)).service_name,
        "mu-desk"
    );
}

fn base_strategy() -> impl Strategy<Value = String> {
    (
        "[a-z]{1,8}",
        proptest::option::of("(/[a-z]{1,6}){1,3}"),
        0..3usize,
    )
        .prop_map(|(host, path, slashes)| {
            format!(
                "http://{host}:4318{}{}",
                path.unwrap_or_default(),
                "/".repeat(slashes)
            )
        })
}

proptest! {
    #[test]
    fn joined_url_is_the_trimmed_base_and_the_path(base in base_strategy()) {
        let export = on(&[], Some(&table(Some(&base))));
        let trimmed = base.trim_end_matches('/');
        prop_assert_eq!(export.traces.unwrap().endpoint, format!("{trimmed}/v1/traces"));
        prop_assert_eq!(export.metrics.unwrap().endpoint, format!("{trimmed}/v1/metrics"));
    }

    #[test]
    fn set_step_wins_over_each_lower_step(
        file in proptest::option::of(base_strategy()),
        base in proptest::option::of(base_strategy()),
        traces in proptest::option::of(base_strategy()),
    ) {
        let mut vars = Vec::new();
        if let Some(base) = &base {
            vars.push((ENDPOINT, base.as_str()));
        }
        if let Some(traces) = &traces {
            vars.push((TRACES_ENDPOINT, traces.as_str()));
        }
        let got = run(&vars, Some(&table(file.as_deref()))).unwrap();
        let expected = traces.clone().or_else(|| {
            base.as_ref()
                .or(file.as_ref())
                .map(|base| format!("{}/v1/traces", base.trim_end_matches('/')))
        });
        prop_assert_eq!(got.and_then(|export| export.traces).map(|signal| signal.endpoint), expected);
    }
}
