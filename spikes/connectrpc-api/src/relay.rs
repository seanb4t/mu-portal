//! A byte relay in front of the server. `cut` closes every live
//! connection and keeps the listener open, as a network drop does.

use std::sync::{Arc, Mutex};

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, TcpStream, UnixListener, UnixStream};
use tokio::task::{AbortHandle, JoinSet};

use crate::Endpoint;

#[derive(Clone, Default)]
pub struct Relay {
    live: Arc<Mutex<JoinSet<()>>>,
}

impl Relay {
    /// Accepts on `front` and forwards each connection to `back`.
    pub fn spawn(self, front: Front, back: Endpoint) -> AbortHandle {
        tokio::spawn(async move {
            loop {
                let accepted = match &front {
                    Front::Tcp(l) => l.accept().await.map(|(s, _)| Conn::Tcp(s)),
                    Front::Unix(l) => l.accept().await.map(|(s, _)| Conn::Unix(s)),
                };
                let Ok(client) = accepted else { return };
                let back = back.clone();
                self.live.lock().unwrap().spawn(async move {
                    let Ok(server) = Conn::connect(&back).await else {
                        return;
                    };
                    pipe(client, server).await;
                });
            }
        })
        .abort_handle()
    }

    /// Closes every connection open now. New connections still go through.
    pub fn cut(&self) -> usize {
        let mut live = self.live.lock().unwrap();
        let n = live.len();
        live.abort_all();
        n
    }
}

pub enum Front {
    Tcp(TcpListener),
    Unix(UnixListener),
}

enum Conn {
    Tcp(TcpStream),
    Unix(UnixStream),
}

impl Conn {
    async fn connect(to: &Endpoint) -> std::io::Result<Self> {
        Ok(match to {
            Endpoint::Tcp(addr) => Conn::Tcp(TcpStream::connect(addr).await?),
            Endpoint::Unix(path) => Conn::Unix(UnixStream::connect(path).await?),
        })
    }
}

async fn pipe(a: Conn, b: Conn) {
    match (a, b) {
        (Conn::Tcp(mut a), Conn::Tcp(mut b)) => copy(&mut a, &mut b).await,
        (Conn::Unix(mut a), Conn::Unix(mut b)) => copy(&mut a, &mut b).await,
        (Conn::Tcp(mut a), Conn::Unix(mut b)) => copy(&mut a, &mut b).await,
        (Conn::Unix(mut a), Conn::Tcp(mut b)) => copy(&mut a, &mut b).await,
    }
}

async fn copy<A, B>(a: &mut A, b: &mut B)
where
    A: AsyncRead + AsyncWrite + Unpin,
    B: AsyncRead + AsyncWrite + Unpin,
{
    let _ = tokio::io::copy_bidirectional(a, b).await;
}
