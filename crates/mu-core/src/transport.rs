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
            let stream = TcpStream::connect((host.as_str(), port)).await?;
            // A MU sends short lines.
            stream.set_nodelay(true)?;
            Ok(Box::new(stream) as Box<dyn ByteStream>)
        })
    }
}
