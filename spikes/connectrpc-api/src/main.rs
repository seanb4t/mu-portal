//! Serves the spike API on a TCP address and a Unix socket at once.
//!
//! `cargo run -p connectrpc-api -- 127.0.0.1:8095 /tmp/mu-spike.sock`

use connectrpc_api::{Log, serve_tcp, serve_unix};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    let addr = args.next().unwrap_or_else(|| "127.0.0.1:8095".into());
    let path = args.next().unwrap_or_else(|| "/tmp/mu-spike.sock".into());
    let _ = std::fs::remove_file(&path);

    let log = Log::new();
    let tcp = tokio::net::TcpListener::bind(&addr).await?;
    let unix = tokio::net::UnixListener::bind(&path)?;
    println!("listening on http://{addr} and {path}");
    tokio::try_join!(serve_tcp(tcp, log.clone()), serve_unix(unix, log))?;
    Ok(())
}
