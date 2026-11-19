//! Transports: the byte stream of a world.

use std::io;

use futures::future::BoxFuture;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;

use crate::id::WorldId;

/// The byte stream of one connection to a game.
pub trait ByteStream: AsyncRead + AsyncWrite + Send + Unpin + 'static {}

impl<T: AsyncRead + AsyncWrite + Send + Unpin + 'static> ByteStream for T {}

/// Opens a byte stream for a world.
pub trait Transport: Send + Sync + 'static {
    /// Connects to the game of `world`.
    fn connect(&self, world: &WorldId) -> BoxFuture<'static, io::Result<Box<dyn ByteStream>>>;
}

/// The TCP connection to the game at `host` and `port`. Each connect resolves the host again.
#[derive(Clone, Debug)]
pub struct TcpTransport {
    host: String,
    port: u16,
}

impl TcpTransport {
    /// A transport to `host` and `port`.
    #[must_use]
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }
}

impl Transport for TcpTransport {
    fn connect(&self, _world: &WorldId) -> BoxFuture<'static, io::Result<Box<dyn ByteStream>>> {
        let host = self.host.clone();
        let port = self.port;
        Box::pin(async move {
            // One future resolves the name and connects, so the timeout of a connect covers both.
            let stream = TcpStream::connect((host.as_str(), port))
                .await
                .map_err(|error| io::Error::new(error.kind(), format!("{host}:{port}: {error}")))?;
            // A MU sends short lines. Without the option the connection still works, only slower.
            if let Err(error) = stream.set_nodelay(true) {
                tracing::warn!(%error, "{host}:{port}: TCP_NODELAY is not set");
            }
            Ok(Box::new(stream) as Box<dyn ByteStream>)
        })
    }
}
