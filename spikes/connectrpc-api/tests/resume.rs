//! A client reads the stream, loses the connection, and resumes from the
//! next sequence number with no gap and no duplicate.

use connectrpc::client::{ClientConfig, ClientTransport, Http2Connection, HttpClient};
use connectrpc::{ConnectError, Protocol};
use connectrpc_api::relay::{Front, Relay};
use connectrpc_api::{Endpoint, Log, PortalServiceClient, SendRequest, SubscribeRequest};
use tokio::net::{TcpListener, UnixListener};

struct Rig {
    log: Log,
    relay: Relay,
    front: Endpoint,
    _dir: tempfile::TempDir,
}

async fn rig_tcp() -> Rig {
    let log = Log::new();
    let server = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let back = Endpoint::Tcp(server.local_addr().unwrap());
    tokio::spawn(connectrpc_api::serve_tcp(server, log.clone()));
    let front = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let front_ep = Endpoint::Tcp(front.local_addr().unwrap());
    let relay = Relay::default();
    relay.clone().spawn(Front::Tcp(front), back);
    Rig {
        log,
        relay,
        front: front_ep,
        _dir: tempfile::tempdir().unwrap(),
    }
}

async fn rig_unix() -> Rig {
    let dir = tempfile::tempdir().unwrap();
    let log = Log::new();
    let back_path = dir.path().join("server.sock");
    let server = UnixListener::bind(&back_path).unwrap();
    tokio::spawn(connectrpc_api::serve_unix(server, log.clone()));
    let front_path = dir.path().join("relay.sock");
    let front = UnixListener::bind(&front_path).unwrap();
    let relay = Relay::default();
    relay
        .clone()
        .spawn(Front::Unix(front), Endpoint::Unix(back_path));
    Rig {
        log,
        relay,
        front: Endpoint::Unix(front_path),
        _dir: dir,
    }
}

fn base(rig: &Rig) -> http::Uri {
    match &rig.front {
        Endpoint::Tcp(addr) => format!("http://{addr}").parse().unwrap(),
        Endpoint::Unix(_) => "http://localhost".parse().unwrap(),
    }
}

async fn send<T>(client: &PortalServiceClient<T>, text: &str) -> u64
where
    T: ClientTransport,
    T::ResponseBody: Unpin,
    <T::ResponseBody as http_body::Body>::Error: std::fmt::Display,
{
    let request = SendRequest {
        text: text.into(),
        ..Default::default()
    };
    client.send(request).await.unwrap().into_owned().seq
}

/// Runs the whole scenario and returns the error the cut gave the client.
async fn cut_and_resume<T>(rig: &Rig, client: PortalServiceClient<T>) -> ConnectError
where
    T: ClientTransport,
    T::ResponseBody: Unpin,
    <T::ResponseBody as http_body::Body>::Error: std::fmt::Display,
{
    for (i, text) in ["one", "two", "three"].iter().enumerate() {
        assert_eq!(send(&client, text).await, i as u64 + 1);
    }

    let mut stream = client
        .subscribe(SubscribeRequest {
            from_seq: 1,
            ..Default::default()
        })
        .await
        .unwrap();
    let mut seen = Vec::new();
    while seen.len() < 3 {
        let event = stream.message().await.unwrap().expect("stream ended early");
        seen.push((event.view().seq, event.view().text.to_string()));
    }
    assert_eq!(*rig.log.subscribers().borrow(), 1);

    assert!(rig.relay.cut() >= 1, "no connection to cut");
    let cut = stream
        .message()
        .await
        .expect_err("a cut must not look like a clean end");

    rig.log.subscribers().wait_for(|n| *n == 0).await.unwrap();

    // Events that arrive while the client is away.
    for text in ["four", "five"] {
        rig.log.append(text.into());
    }

    let last = seen.last().unwrap().0;
    let mut stream = client
        .subscribe(SubscribeRequest {
            from_seq: last + 1,
            ..Default::default()
        })
        .await
        .unwrap();
    for _ in 0..2 {
        let event = stream.message().await.unwrap().unwrap();
        seen.push((event.view().seq, event.view().text.to_string()));
    }
    assert_eq!(send(&client, "six").await, 6);
    let event = stream.message().await.unwrap().unwrap();
    seen.push((event.view().seq, event.view().text.to_string()));

    let want: Vec<(u64, String)> = ["one", "two", "three", "four", "five", "six"]
        .iter()
        .enumerate()
        .map(|(i, t)| (i as u64 + 1, t.to_string()))
        .collect();
    assert_eq!(seen, want);
    cut
}

fn report(name: &str, rig: &Rig, cut: &ConnectError) {
    println!(
        "{name}: cut -> {:?} {:?}; versions {:?}",
        cut.code,
        cut.message,
        rig.log.http_versions()
    );
}

#[tokio::test]
async fn unix_h2c_connect() {
    let rig = rig_unix().await;
    let Endpoint::Unix(path) = rig.front.clone() else {
        unreachable!()
    };
    let conn = Http2Connection::lazy_unix(path, base(&rig)).shared(1024);
    let client = PortalServiceClient::new(conn, ClientConfig::new(base(&rig)));
    let cut = cut_and_resume(&rig, client).await;
    report("unix_h2c_connect", &rig, &cut);
}

#[tokio::test]
async fn unix_h2c_grpc() {
    let rig = rig_unix().await;
    let Endpoint::Unix(path) = rig.front.clone() else {
        unreachable!()
    };
    let conn = Http2Connection::lazy_unix(path, base(&rig)).shared(1024);
    let config = ClientConfig::new(base(&rig)).with_protocol(Protocol::Grpc);
    let cut = cut_and_resume(&rig, PortalServiceClient::new(conn, config)).await;
    report("unix_h2c_grpc", &rig, &cut);
}

#[tokio::test]
async fn tcp_http1_connect() {
    let rig = rig_tcp().await;
    let client = PortalServiceClient::new(HttpClient::plaintext(), ClientConfig::new(base(&rig)));
    let cut = cut_and_resume(&rig, client).await;
    report("tcp_http1_connect", &rig, &cut);
}

#[tokio::test]
async fn tcp_h2c_connect() {
    let rig = rig_tcp().await;
    let http = HttpClient::plaintext_http2_only();
    let client = PortalServiceClient::new(http, ClientConfig::new(base(&rig)));
    let cut = cut_and_resume(&rig, client).await;
    report("tcp_h2c_connect", &rig, &cut);
}

#[tokio::test]
async fn tcp_h2c_grpc() {
    let rig = rig_tcp().await;
    let http = HttpClient::plaintext_http2_only();
    let config = ClientConfig::new(base(&rig)).with_protocol(Protocol::Grpc);
    let cut = cut_and_resume(&rig, PortalServiceClient::new(http, config)).await;
    report("tcp_h2c_grpc", &rig, &cut);
}

#[tokio::test]
async fn tcp_http1_grpc_web() {
    let rig = rig_tcp().await;
    let config = ClientConfig::new(base(&rig)).with_protocol(Protocol::GrpcWeb);
    let client = PortalServiceClient::new(HttpClient::plaintext(), config);
    let cut = cut_and_resume(&rig, client).await;
    report("tcp_http1_grpc_web", &rig, &cut);
}
