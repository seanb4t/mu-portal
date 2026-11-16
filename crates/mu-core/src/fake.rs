//! The fake game: a game in the process that stands in for a real one in tests and
//! behind the `--fake-world` flag.

use std::io;
use std::sync::{Arc, Mutex, PoisonError};

use futures::FutureExt as _;
use futures::future::BoxFuture;
use tokio::io::{AsyncBufReadExt as _, AsyncWriteExt as _, BufReader, DuplexStream};
use tokio::sync::{mpsc, watch};

use crate::id::WorldId;
use crate::transport::{ByteStream, Transport};

/// The answer of the fake game to a wrong login.
pub const WRONG_LOGIN: &str = "Either that player does not exist, or has a different password.";

const BUFFER: usize = 64 * 1024;

/// What the fake game accepts.
#[derive(Clone)]
struct Account {
    login: String,
    password: String,
}

/// The lines that the fake game got, and a way to make it send a line.
#[derive(Clone)]
pub struct FakeGame {
    received: Arc<watch::Sender<Vec<String>>>,
    say: Arc<Mutex<Option<mpsc::UnboundedSender<String>>>>,
    connects: Arc<watch::Sender<usize>>,
}

impl FakeGame {
    fn new() -> Self {
        Self {
            received: Arc::new(watch::channel(Vec::new()).0),
            say: Arc::default(),
            connects: Arc::new(watch::channel(0).0),
        }
    }

    /// Every line that the game got, the login line included, in order.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        self.received.borrow().clone()
    }

    /// Waits until the game got `count` lines in all.
    pub async fn wait_for_lines(&self, count: usize) -> Vec<String> {
        let mut receiver = self.received.subscribe();
        receiver
            .wait_for(|lines| lines.len() >= count)
            .await
            .map_or_else(|_| Vec::new(), |lines| lines.clone())
    }

    /// Waits until the transport opened `count` connections in all.
    pub async fn wait_for_connects(&self, count: usize) {
        let mut receiver = self.connects.subscribe();
        let _ = receiver.wait_for(|n| *n >= count).await;
    }

    /// Makes the game send `line` on the open connection. Returns false with no connection.
    pub fn say(&self, line: &str) -> bool {
        let say = self.say.lock().unwrap_or_else(PoisonError::into_inner);
        say.as_ref()
            .is_some_and(|sender| sender.send(line.to_owned()).is_ok())
    }
}

/// Runs the fake game on one stream until the stream or the game closes it.
///
/// It checks the login line `connect <login> <password>` and greets with
/// `Welcome, <login>.`. After the login it answers each line `<text>` with
/// `You say, "<text>"`, and closes on `@drop`.
async fn serve(stream: DuplexStream, account: Account, game: FakeGame) -> io::Result<()> {
    let (read, mut write) = tokio::io::split(stream);
    let mut lines = BufReader::new(read).lines();
    let (say, mut said) = mpsc::unbounded_channel();
    *game.say.lock().unwrap_or_else(PoisonError::into_inner) = Some(say);
    let expected = format!("connect {} {}", account.login, account.password);
    let mut logged_in = false;
    loop {
        let line = tokio::select! {
            line = lines.next_line() => line?,
            Some(text) = said.recv() => {
                write.write_all(format!("{text}\r\n").as_bytes()).await?;
                continue;
            }
        };
        let Some(line) = line else { return Ok(()) };
        let line = line.trim_end_matches('\r').to_owned();
        game.received.send_modify(|lines| lines.push(line.clone()));
        let answer = if !logged_in {
            if line == expected {
                logged_in = true;
                format!("Welcome, {}.", account.login)
            } else {
                WRONG_LOGIN.to_owned()
            }
        } else if line == "@drop" {
            write.shutdown().await?;
            return Ok(());
        } else {
            format!("You say, \"{line}\"")
        };
        write.write_all(format!("{answer}\r\n").as_bytes()).await?;
    }
}

/// A transport whose every connection reaches a new run of the fake game.
pub struct FakeTransport {
    account: Account,
    game: FakeGame,
    gate: Option<watch::Receiver<bool>>,
}

/// Lets a held connect of a [`FakeTransport`] go on.
pub struct ConnectGate(watch::Sender<bool>);

impl ConnectGate {
    /// Opens the gate for every connect, now and later.
    pub fn open(&self) {
        self.0.send_replace(true);
    }
}

impl FakeTransport {
    /// A fake game that accepts `login` with `password`.
    #[must_use]
    pub fn new(login: &str, password: &str) -> Self {
        Self {
            account: Account {
                login: login.to_owned(),
                password: password.to_owned(),
            },
            game: FakeGame::new(),
            gate: None,
        }
    }

    /// The same transport, with each connect held until the gate opens.
    #[must_use]
    pub fn held(mut self) -> (Self, ConnectGate) {
        let (sender, receiver) = watch::channel(false);
        self.gate = Some(receiver);
        (self, ConnectGate(sender))
    }

    /// The view of the game for a test.
    #[must_use]
    pub fn game(&self) -> FakeGame {
        self.game.clone()
    }
}

impl Transport for FakeTransport {
    fn connect(&self, _world: &WorldId) -> BoxFuture<'static, io::Result<Box<dyn ByteStream>>> {
        let account = self.account.clone();
        let game = self.game.clone();
        let held = self.gate.clone();
        async move {
            if let Some(mut gate) = held {
                gate.wait_for(|open| *open)
                    .await
                    .map_err(|_| io::Error::other("the connect gate closed"))?;
            }
            let (ours, theirs) = tokio::io::duplex(BUFFER);
            game.connects.send_modify(|n| *n += 1);
            tokio::spawn(serve(theirs, account, game));
            Ok(Box::new(ours) as Box<dyn ByteStream>)
        }
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn open() -> (FakeGame, tokio::io::Lines<BufReader<Box<dyn ByteStream>>>) {
        let transport = FakeTransport::new("aria", "s3cret-pw");
        let game = transport.game();
        let stream = transport
            .connect(&"alpha/aria".parse().unwrap())
            .await
            .unwrap();
        (game, BufReader::new(stream).lines())
    }

    async fn send(lines: &mut tokio::io::Lines<BufReader<Box<dyn ByteStream>>>, line: &str) {
        lines
            .get_mut()
            .write_all(format!("{line}\r\n").as_bytes())
            .await
            .unwrap();
    }

    async fn next(lines: &mut tokio::io::Lines<BufReader<Box<dyn ByteStream>>>) -> Option<String> {
        lines
            .next_line()
            .await
            .unwrap()
            .map(|line| line.trim_end_matches('\r').to_owned())
    }

    #[tokio::test]
    async fn fake_game_accepts_login() {
        let (_, mut lines) = open().await;
        send(&mut lines, "connect aria s3cret-pw").await;
        assert_eq!(next(&mut lines).await.as_deref(), Some("Welcome, aria."));
    }

    #[tokio::test]
    async fn fake_game_refuses_wrong_password() {
        let (_, mut lines) = open().await;
        send(&mut lines, "connect aria nope").await;
        assert_eq!(next(&mut lines).await.as_deref(), Some(WRONG_LOGIN));
        send(&mut lines, "hello").await;
        assert_eq!(next(&mut lines).await.as_deref(), Some(WRONG_LOGIN));
    }

    #[tokio::test]
    async fn fake_game_answers_line() {
        let (game, mut lines) = open().await;
        send(&mut lines, "connect aria s3cret-pw").await;
        next(&mut lines).await;
        send(&mut lines, "hello").await;
        assert_eq!(
            next(&mut lines).await.as_deref(),
            Some("You say, \"hello\"")
        );
        assert_eq!(game.lines(), ["connect aria s3cret-pw", "hello"]);
    }

    #[tokio::test]
    async fn fake_game_drops_on_request() {
        let (_, mut lines) = open().await;
        send(&mut lines, "connect aria s3cret-pw").await;
        next(&mut lines).await;
        send(&mut lines, "@drop").await;
        assert_eq!(next(&mut lines).await, None);
    }

    #[tokio::test]
    async fn fake_game_says_a_line() {
        let (game, mut lines) = open().await;
        game.wait_for_connects(1).await;
        send(&mut lines, "connect aria s3cret-pw").await;
        next(&mut lines).await;
        assert!(game.say("buy spam now"));
        assert_eq!(next(&mut lines).await.as_deref(), Some("buy spam now"));
    }

    #[tokio::test(start_paused = true)]
    async fn held_connect_waits_for_the_gate() {
        let (transport, gate) = FakeTransport::new("aria", "pw").held();
        let mut connect = transport.connect(&"alpha/aria".parse().unwrap());
        assert!(futures::poll!(&mut connect).is_pending());
        gate.open();
        assert!(connect.await.is_ok());
    }
}
