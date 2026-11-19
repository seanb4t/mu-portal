//! The scripted fake game. It runs a script of raw bytes on each connection, with no framer
//! of the engine, over TCP or over an in-memory duplex stream.

use std::io;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use futures::FutureExt as _;
use futures::future::BoxFuture;
use mu_core::id::WorldId;
use mu_core::transport::{ByteStream, Transport};
use tokio::io::{AsyncRead, AsyncReadExt as _, AsyncWrite, AsyncWriteExt as _, ReadHalf};
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::Instant;

const BUFFER: usize = 64 * 1024;

/// One step of a [`Script`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// Writes the bytes to the client.
    Send(Vec<u8>),
    /// Waits until the client writes the bytes. The search starts after the last match.
    WaitFor(Vec<u8>),
    /// Sends nothing for the time. The time moves with the paused clock of tokio.
    SilentFor(Duration),
    /// Sends nothing until the client closes the connection.
    Silent,
    /// Closes the connection.
    Close,
}

/// The steps that the game runs on each connection. After the last step, the game sends
/// nothing until the client closes the connection.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Script(Vec<Step>);

impl Script {
    /// A script with no step.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A script that replays the reads of a recorded transcript: one [`Step::Send`] for each
    /// read, in order. The caller parses the transcript and gives the reads of the game.
    #[must_use]
    pub fn replay(reads: impl IntoIterator<Item = impl AsRef<[u8]>>) -> Self {
        reads.into_iter().fold(Self::new(), Self::send)
    }

    /// Adds a [`Step::Send`].
    #[must_use]
    pub fn send(self, bytes: impl AsRef<[u8]>) -> Self {
        self.step(Step::Send(bytes.as_ref().to_vec()))
    }

    /// Adds a [`Step::WaitFor`].
    #[must_use]
    pub fn wait_for(self, bytes: impl AsRef<[u8]>) -> Self {
        self.step(Step::WaitFor(bytes.as_ref().to_vec()))
    }

    /// Adds a [`Step::SilentFor`].
    #[must_use]
    pub fn silent_for(self, time: Duration) -> Self {
        self.step(Step::SilentFor(time))
    }

    /// Adds a [`Step::Silent`].
    #[must_use]
    pub fn silent(self) -> Self {
        self.step(Step::Silent)
    }

    /// Adds a [`Step::Close`].
    #[must_use]
    pub fn close(self) -> Self {
        self.step(Step::Close)
    }

    /// Adds a step.
    #[must_use]
    pub fn step(mut self, step: Step) -> Self {
        self.0.push(step);
        self
    }
}

/// What the client wrote on one connection.
#[derive(Clone, Debug)]
struct Connection {
    bytes: Vec<u8>,
    /// Each read of the game, with its time.
    reads: Vec<(Instant, Vec<u8>)>,
    open: bool,
}

struct Shared {
    script: Script,
    connections: watch::Sender<Vec<Connection>>,
    refused: watch::Sender<usize>,
    to_refuse: Mutex<usize>,
}

/// A fake game that runs the same [`Script`] on each connection, and records every byte that
/// the client wrote.
///
/// As a [`Transport`], each connect gets a new in-memory duplex stream, and
/// [`ScriptedGame::refuse`] can refuse a connect. [`ScriptedGame::listen`] serves the script
/// over TCP instead. A real socket does not move with the paused clock of tokio, so the
/// tests on the paused clock use the transport.
#[derive(Clone)]
pub struct ScriptedGame {
    shared: Arc<Shared>,
}

impl ScriptedGame {
    /// A game that runs `script` on each connection.
    #[must_use]
    pub fn new(script: Script) -> Self {
        Self {
            shared: Arc::new(Shared {
                script,
                connections: watch::channel(Vec::new()).0,
                refused: watch::channel(0).0,
                to_refuse: Mutex::new(0),
            }),
        }
    }

    /// Refuses the next `count` connects through the transport, with
    /// [`io::ErrorKind::ConnectionRefused`]. A TCP test gets a refused connect from a port that
    /// nothing listens on.
    pub fn refuse(&self, count: usize) {
        *self.lock_to_refuse() += count;
    }

    /// The number of connections that the game accepted.
    #[must_use]
    pub fn connects(&self) -> usize {
        self.shared.connections.borrow().len()
    }

    /// The number of connects that the game refused.
    #[must_use]
    pub fn refused(&self) -> usize {
        *self.shared.refused.borrow()
    }

    /// The bytes that the client wrote, one entry for each accepted connection, in order.
    #[must_use]
    pub fn written(&self) -> Vec<Vec<u8>> {
        self.shared
            .connections
            .borrow()
            .iter()
            .map(|connection| connection.bytes.clone())
            .collect()
    }

    /// Each read of the game, with the time of the read, one entry for each accepted connection,
    /// in order. On the paused clock, the time of a read is the time of the write of the client.
    #[must_use]
    pub fn reads(&self) -> Vec<Vec<(Instant, Vec<u8>)>> {
        self.shared
            .connections
            .borrow()
            .iter()
            .map(|connection| connection.reads.clone())
            .collect()
    }

    /// Waits until the game accepted `count` connections in all.
    pub async fn wait_for_connects(&self, count: usize) {
        let mut receiver = self.shared.connections.subscribe();
        // The sender lives in `self`, so the channel stays open.
        let _ = receiver
            .wait_for(|connections| connections.len() >= count)
            .await;
    }

    /// Waits until the game refused `count` connects in all.
    pub async fn wait_for_refused(&self, count: usize) {
        let mut receiver = self.shared.refused.subscribe();
        // The sender lives in `self`, so the channel stays open.
        let _ = receiver.wait_for(|refused| *refused >= count).await;
    }

    /// Waits until `done` holds for the reads of the game, as [`ScriptedGame::reads`] gives them.
    pub async fn wait_reads(&self, done: impl Fn(&[Vec<(Instant, Vec<u8>)>]) -> bool) {
        let mut receiver = self.shared.connections.subscribe();
        // The sender lives in `self`, so the channel stays open.
        let _ = receiver
            .wait_for(|connections| {
                let reads: Vec<_> = connections
                    .iter()
                    .map(|connection| connection.reads.clone())
                    .collect();
                done(&reads)
            })
            .await;
    }

    /// Serves the script over TCP on a free port of `127.0.0.1`.
    ///
    /// # Errors
    ///
    /// Returns the error of the bind.
    pub async fn listen(&self) -> io::Result<Listening> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await?;
        let addr = listener.local_addr()?;
        let game = self.clone();
        let task = tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                game.serve(stream);
            }
        });
        Ok(Listening {
            addr,
            task: Some(task),
        })
    }

    fn lock_to_refuse(&self) -> std::sync::MutexGuard<'_, usize> {
        self.shared
            .to_refuse
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn take_refusal(&self) -> bool {
        let mut to_refuse = self.lock_to_refuse();
        let refuse = *to_refuse > 0;
        if refuse {
            *to_refuse -= 1;
            self.shared.refused.send_modify(|refused| *refused += 1);
        }
        refuse
    }

    /// Records a new connection and runs the script on it.
    fn serve<S>(&self, stream: S)
    where
        S: AsyncRead + AsyncWrite + Send + 'static,
    {
        let mut index = 0;
        self.shared.connections.send_modify(|connections| {
            index = connections.len();
            connections.push(Connection {
                bytes: Vec::new(),
                reads: Vec::new(),
                open: true,
            });
        });
        tokio::spawn(run(stream, Arc::clone(&self.shared), index));
    }
}

impl Transport for ScriptedGame {
    fn connect(&self, _world: &WorldId) -> BoxFuture<'static, io::Result<Box<dyn ByteStream>>> {
        let game = self.clone();
        async move {
            if game.take_refusal() {
                return Err(io::ErrorKind::ConnectionRefused.into());
            }
            let (client, server) = tokio::io::duplex(BUFFER);
            game.serve(server);
            Ok(Box::new(client) as Box<dyn ByteStream>)
        }
        .boxed()
    }
}

/// A [`ScriptedGame`] that listens on TCP. The game stops listening when this drops.
pub struct Listening {
    addr: SocketAddr,
    task: Option<JoinHandle<()>>,
}

impl Listening {
    /// The address that the game listens on.
    #[must_use]
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Stops listening and waits until the port is closed. The open connections stay open.
    pub async fn stop(mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
            // The listener drops before the task gives its result.
            let _ = task.await;
        }
    }
}

impl Drop for Listening {
    fn drop(&mut self) {
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}

async fn run<S>(stream: S, shared: Arc<Shared>, index: usize)
where
    S: AsyncRead + AsyncWrite + Send + 'static,
{
    let (read, mut write) = tokio::io::split(stream);
    let reader = tokio::spawn(record(read, Arc::clone(&shared), index));
    let mut connection = shared.connections.subscribe();
    let mut cursor = 0;
    for step in &shared.script.0 {
        match step {
            Step::Send(bytes) => {
                if write.write_all(bytes).await.is_err() {
                    break;
                }
            }
            Step::WaitFor(needle) => {
                let Some(end) = wait_for_bytes(&mut connection, index, cursor, needle).await else {
                    break;
                };
                cursor = end;
            }
            Step::SilentFor(time) => tokio::time::sleep(*time).await,
            Step::Silent => break,
            Step::Close => {
                let _ = write.shutdown().await;
                reader.abort();
                shared
                    .connections
                    .send_modify(|connections| connections[index].open = false);
                return;
            }
        }
    }
    // The connection stays open, with no more bytes from the game, until the client closes it.
    let _ = connection
        .wait_for(|connections| !connections[index].open)
        .await;
}

/// Reads every byte that the client writes into the record of the connection.
async fn record<S>(mut read: ReadHalf<S>, shared: Arc<Shared>, index: usize)
where
    S: AsyncRead + AsyncWrite,
{
    let mut buffer = vec![0; 4096];
    loop {
        let count = read.read(&mut buffer).await.unwrap_or(0);
        if count == 0 {
            shared
                .connections
                .send_modify(|connections| connections[index].open = false);
            return;
        }
        shared.connections.send_modify(|connections| {
            let connection = &mut connections[index];
            connection.bytes.extend_from_slice(&buffer[..count]);
            connection
                .reads
                .push((Instant::now(), buffer[..count].to_vec()));
        });
    }
}

/// Waits until the client wrote `needle` after `cursor`, and returns the end of the match.
/// Returns `None` if the client closed the connection first.
async fn wait_for_bytes(
    connection: &mut watch::Receiver<Vec<Connection>>,
    index: usize,
    cursor: usize,
    needle: &[u8],
) -> Option<usize> {
    let connections = connection
        .wait_for(|connections| {
            let connection = &connections[index];
            find(&connection.bytes[cursor..], needle).is_some() || !connection.open
        })
        .await
        .ok()?;
    find(&connections[index].bytes[cursor..], needle).map(|start| cursor + start + needle.len())
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use tokio::net::TcpStream;
    use tokio::time::Instant;

    use super::*;

    fn world() -> WorldId {
        "alpha/aria".parse().unwrap()
    }

    async fn connect(game: &ScriptedGame) -> Box<dyn ByteStream> {
        game.connect(&world()).await.unwrap()
    }

    async fn read_exact(stream: &mut (impl AsyncRead + Unpin), count: usize) -> Vec<u8> {
        let mut bytes = vec![0; count];
        stream.read_exact(&mut bytes).await.unwrap();
        bytes
    }

    #[tokio::test(start_paused = true)]
    async fn send_step_writes_the_bytes() {
        let game = ScriptedGame::new(Script::new().send(b"\xff\xfb\x01hello\r\n"));
        let mut stream = connect(&game).await;
        assert_eq!(read_exact(&mut stream, 10).await, b"\xff\xfb\x01hello\r\n");
    }

    #[tokio::test(start_paused = true)]
    async fn wait_step_holds_the_next_step_until_the_bytes_arrive() {
        let game = ScriptedGame::new(Script::new().wait_for(b"go").send(b"ok"));
        let mut stream = connect(&game).await;
        stream.write_all(b"g").await.unwrap();
        let early = tokio::time::timeout(Duration::from_secs(60), read_exact(&mut stream, 2)).await;
        assert!(
            early.is_err(),
            "the game sent before the client wrote \"go\""
        );
        stream.write_all(b"o").await.unwrap();
        assert_eq!(read_exact(&mut stream, 2).await, b"ok");
    }

    #[tokio::test(start_paused = true)]
    async fn wait_step_searches_after_the_last_match() {
        let game = ScriptedGame::new(
            Script::new()
                .wait_for(b"x")
                .send(b"1")
                .wait_for(b"x")
                .send(b"2"),
        );
        let mut stream = connect(&game).await;
        stream.write_all(b"x").await.unwrap();
        assert_eq!(read_exact(&mut stream, 1).await, b"1");
        let early = tokio::time::timeout(Duration::from_secs(60), read_exact(&mut stream, 1)).await;
        assert!(early.is_err(), "one \"x\" matched two waits");
        stream.write_all(b"x").await.unwrap();
        assert_eq!(read_exact(&mut stream, 1).await, b"2");
    }

    #[tokio::test(start_paused = true)]
    async fn silent_for_step_sends_nothing_for_the_time() {
        let game = ScriptedGame::new(
            Script::new()
                .send(b"a")
                .silent_for(Duration::from_secs(5))
                .send(b"b"),
        );
        let mut stream = connect(&game).await;
        assert_eq!(read_exact(&mut stream, 1).await, b"a");
        let start = Instant::now();
        assert_eq!(read_exact(&mut stream, 1).await, b"b");
        assert_eq!(start.elapsed(), Duration::from_secs(5));
    }

    #[tokio::test(start_paused = true)]
    async fn silent_step_keeps_the_connection_open_with_no_bytes() {
        let game = ScriptedGame::new(Script::new().send(b"a").silent().send(b"b"));
        let mut stream = connect(&game).await;
        assert_eq!(read_exact(&mut stream, 1).await, b"a");
        let mut byte = [0; 1];
        let read = tokio::time::timeout(Duration::from_secs(3600), stream.read(&mut byte)).await;
        assert!(read.is_err(), "the game sent a byte or closed: {read:?}");
    }

    #[tokio::test(start_paused = true)]
    async fn close_step_closes_the_connection() {
        let game = ScriptedGame::new(Script::new().send(b"bye").close());
        let mut stream = connect(&game).await;
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes).await.unwrap();
        assert_eq!(bytes, b"bye");
    }

    #[test]
    fn replay_sends_each_read_in_order() {
        let reads: [&[u8]; 2] = [b"\xff\xfb\x01", b"Welcome\r\n"];
        assert_eq!(
            Script::replay(reads),
            Script::new().send(b"\xff\xfb\x01").send(b"Welcome\r\n")
        );
    }

    #[tokio::test(start_paused = true)]
    async fn refused_connect_fails_and_is_counted() {
        let game = ScriptedGame::new(Script::new());
        game.refuse(1);
        let refused = game.connect(&world()).await.err().map(|error| error.kind());
        assert_eq!(refused, Some(io::ErrorKind::ConnectionRefused));
        let _stream = connect(&game).await;
        assert_eq!((game.refused(), game.connects()), (1, 1));
    }

    #[tokio::test(start_paused = true)]
    async fn written_records_the_bytes_of_each_connection() {
        let game = ScriptedGame::new(Script::new().wait_for(b"\r\n").send(b"ok"));
        let mut first = connect(&game).await;
        first.write_all(b"connect aria ").await.unwrap();
        first.write_all(b"pw\r\n").await.unwrap();
        read_exact(&mut first, 2).await;
        let mut second = connect(&game).await;
        second.write_all(b"\xff\xf4\r\n").await.unwrap();
        read_exact(&mut second, 2).await;
        assert_eq!(
            game.written(),
            vec![b"connect aria pw\r\n".to_vec(), b"\xff\xf4\r\n".to_vec()]
        );
    }

    #[tokio::test(start_paused = true)]
    async fn reads_carry_their_time() {
        let game = ScriptedGame::new(Script::new());
        let start = Instant::now();
        let mut stream = connect(&game).await;
        for (bytes, count) in [(b"one", 1), (b"two", 2)] {
            if count == 2 {
                tokio::time::advance(Duration::from_secs(5)).await;
            }
            stream.write_all(bytes).await.unwrap();
            game.wait_reads(|reads| reads[0].len() == count).await;
        }
        let reads: Vec<(Duration, Vec<u8>)> = game.reads()[0]
            .iter()
            .map(|(time, bytes)| (*time - start, bytes.clone()))
            .collect();
        assert_eq!(
            reads,
            [
                (Duration::ZERO, b"one".to_vec()),
                (Duration::from_secs(5), b"two".to_vec())
            ]
        );
    }

    #[tokio::test]
    async fn listen_runs_the_script_over_tcp() {
        let game = ScriptedGame::new(Script::new().send(b"hi").wait_for(b"q").close());
        let listening = game.listen().await.unwrap();
        let mut stream = TcpStream::connect(listening.addr()).await.unwrap();
        assert_eq!(read_exact(&mut stream, 2).await, b"hi");
        stream.write_all(b"q").await.unwrap();
        let mut rest = Vec::new();
        stream.read_to_end(&mut rest).await.unwrap();
        assert_eq!((rest, game.written()), (Vec::new(), vec![b"q".to_vec()]));
    }

    #[tokio::test]
    async fn stopped_listener_refuses_a_connect() {
        let game = ScriptedGame::new(Script::new());
        let listening = game.listen().await.unwrap();
        let addr = listening.addr();
        listening.stop().await;
        let refused = TcpStream::connect(addr)
            .await
            .err()
            .map(|error| error.kind());
        assert_eq!(refused, Some(io::ErrorKind::ConnectionRefused));
    }
}
