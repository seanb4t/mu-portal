//! Joins a tailnet and serves the spike API of `connectrpc-api` on it.
//!
//! `TS_AUTH_KEY=... cargo run -p embedded-tailscale -- <state-dir> [hostname] [port]`
//!
//! Prints the identity of the node as one line, then serves until Ctrl-C.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::{ConnectInfo, Request, State};
use axum::middleware::Next;
use axum::response::Response;

use connectrpc_api::Log;
use embedded_tailscale::{Caller, JoinConfig, Node};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()),
        )
        .init();

    let mut args = std::env::args().skip(1);
    let state_dir = args
        .next()
        .ok_or("usage: embedded-tailscale <state-dir> [hostname] [port]")?;
    let hostname = args.next().unwrap_or_else(|| "mu-spike".into());
    let port: u16 = args.next().map(|p| p.parse()).transpose()?.unwrap_or(8095);

    let node = Node::join(JoinConfig {
        state_dir: state_dir.into(),
        hostname,
        auth_key: std::env::var("TS_AUTH_KEY").ok().filter(|k| !k.is_empty()),
    })
    .await?;
    println!("identity {:?}", node.identity().await?);

    let listener = node.listen(port).await?;
    println!("listening on tailnet port {port}");
    let node = Arc::new(node);
    let app = connectrpc_api::app(Log::new()).layer(axum::middleware::from_fn_with_state(
        node.clone(),
        name_caller,
    ));
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<Caller>(),
    )
    .with_graceful_shutdown(async {
        let _ = tokio::signal::ctrl_c().await;
    })
    .await?;

    let node = Arc::into_inner(node).ok_or("the node is still shared")?;
    let clean = node.leave(Duration::from_secs(5)).await;
    println!("left the tailnet, clean: {clean}");
    Ok(())
}

async fn name_caller(
    State(node): State<Arc<Node>>,
    ConnectInfo(Caller(addr)): ConnectInfo<Caller>,
    req: Request,
    next: Next,
) -> Response {
    let peer = node.peer_name(addr.ip()).await.ok().flatten();
    println!(
        "{} {} from {} ({})",
        req.method(),
        req.uri().path(),
        addr.ip(),
        peer.as_deref().unwrap_or("unknown peer")
    );
    next.run(req).await
}
