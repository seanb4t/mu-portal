//! Transports: the byte stream of a world. #54 adds TCP and #58 TLS.

use std::io;

use futures::future::BoxFuture;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::id::WorldId;

/// The byte stream of one connection to a game.
pub trait ByteStream: AsyncRead + AsyncWrite + Send + Unpin + 'static {}

impl<T: AsyncRead + AsyncWrite + Send + Unpin + 'static> ByteStream for T {}

/// Opens a byte stream for a world.
pub trait Transport: Send + Sync + 'static {
    /// Connects to the game of `world`.
    fn connect(&self, world: &WorldId) -> BoxFuture<'static, io::Result<Box<dyn ByteStream>>>;
}
