//! Helpers that run the built binary as a server, relay its socket, or give it a
//! pseudo-terminal on one stream.

use std::ffi::OsString;
use std::fmt::Display;
use std::fs::File;
use std::future::Future;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Output, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use mu_client::PortalClient;
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt as _, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::watch;
use tokio::task::JoinSet;

/// The bound of each wait in the tests of the binary.
pub const BOUND: Duration = Duration::from_secs(20);

pub use mu_server::SERVING;

/// The line that the fake game sends after the login of `world`.
pub fn greeting(world: &str) -> String {
    let world: mu_core::WorldId = world.parse().unwrap();
    format!("Welcome, {}.", world.character())
}

/// The password of each fake world in the tests.
pub const PW: &str = "s3cret-pw";

/// The fake worlds `alpha/aria` and `beta/bryn`, each with [`PW`].
pub const TWO_WORLDS: &[&str] = &["alpha/aria", "beta/bryn"];

/// A time past the first attempt of the default reconnect, which comes 1 second after a drop,
/// plus at most 5 percent.
pub const PAST_FIRST_WAIT: Duration = Duration::from_secs(3);

/// The socket path and the process id of a serving line.
pub fn served(line: &str) -> Option<(PathBuf, u32)> {
    let rest = line.strip_prefix(SERVING)?.strip_prefix(' ')?;
    let (path, process) = rest.rsplit_once(" (process ")?;
    Some((
        PathBuf::from(path),
        process.strip_suffix(')')?.parse().ok()?,
    ))
}

/// Sends SIGTERM on drop to each server that a client started, as `server.log` names it.
pub struct Reaper(pub PathBuf);

impl Drop for Reaper {
    fn drop(&mut self) {
        let log = std::fs::read_to_string(&self.0).unwrap_or_default();
        for (_, pid) in log.lines().filter_map(served) {
            if let Some(pid) = rustix::process::Pid::from_raw(pid.try_into().unwrap()) {
                // A server that stopped already is not an error here.
                let _ = rustix::process::kill_process(pid, rustix::process::Signal::TERM);
            }
        }
    }
}

/// The one bounded wait: `future`, or a failed test at `bound` that names `what`.
pub async fn wait_until<T>(
    bound: Duration,
    what: impl Display,
    future: impl Future<Output = T>,
) -> T {
    tokio::time::timeout(bound, future)
        .await
        .unwrap_or_else(|elapsed| panic!("no {what} within {bound:?}: {elapsed}"))
}

/// A temporary directory with mode 0700, which a socket can go in.
pub fn private_tempdir() -> TempDir {
    use std::os::unix::fs::PermissionsExt as _;
    tempfile::Builder::new()
        .permissions(std::fs::Permissions::from_mode(0o700))
        .tempdir()
        .unwrap()
}

/// The lines of stderr, and whether stderr closed.
#[derive(Default)]
struct Stderr {
    lines: Vec<String>,
    closed: bool,
}

/// The arguments and the environment of a `mu server` to start.
pub struct ServerBuilder {
    dir: Arc<TempDir>,
    socket_flag: Option<PathBuf>,
    current_dir: Option<PathBuf>,
    args: Vec<OsString>,
    env: Vec<(OsString, Option<OsString>)>,
}

impl ServerBuilder {
    /// A server with `--socket <dir>/s.sock` in a new temporary directory, with
    /// [`super::test_env`].
    pub fn new() -> Self {
        let dir = Arc::new(private_tempdir());
        let socket_flag = Some(dir.path().join("s.sock"));
        Self {
            dir,
            socket_flag,
            current_dir: None,
            args: Vec::new(),
            env: super::test_env(),
        }
    }

    /// The same server in the directory of another server.
    pub fn in_dir(mut self, dir: &Arc<TempDir>) -> Self {
        self.dir = Arc::clone(dir);
        self.socket_flag = Some(dir.path().join("s.sock"));
        self
    }

    /// The flag `--socket` with `path`, or no flag.
    pub fn socket_flag(mut self, path: Option<PathBuf>) -> Self {
        self.socket_flag = path;
        self
    }

    /// Runs the server in `dir`.
    pub fn current_dir(mut self, dir: &Path) -> Self {
        self.current_dir = Some(dir.to_owned());
        self
    }

    /// Adds one argument after `server`.
    pub fn arg(mut self, arg: impl Into<OsString>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Adds `--fake-world <id>=env:<variable>` and sets the variable to `password`.
    pub fn fake_world(self, id: &str, variable: &str, password: &str) -> Self {
        self.arg("--fake-world")
            .arg(format!("{id}=env:{variable}"))
            .env(variable, Some(password))
    }

    /// Adds each id as a fake world with the password [`PW`], in the variables `PW_0`, `PW_1`
    /// and so on.
    pub fn fake_worlds(self, worlds: &[&str]) -> Self {
        worlds.iter().enumerate().fold(self, |builder, (n, world)| {
            builder.fake_world(world, &format!("PW_{n}"), PW)
        })
    }

    /// Sets a variable, or removes it with `None`.
    pub fn env(mut self, name: impl Into<OsString>, value: Option<impl Into<OsString>>) -> Self {
        self.env.push((name.into(), value.map(Into::into)));
        self
    }

    /// The temporary directory of the server.
    pub fn dir(&self) -> &Path {
        self.dir.path()
    }

    fn command(&self) -> tokio::process::Command {
        let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_mu"));
        command.arg("server");
        if let Some(socket) = &self.socket_flag {
            command.arg("--socket").arg(socket);
        }
        command.args(&self.args);
        if let Some(dir) = &self.current_dir {
            command.current_dir(dir);
        }
        super::apply_env(command.as_std_mut(), &self.env);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        command.kill_on_drop(true);
        command
    }

    /// Starts the server and waits until it serves.
    pub async fn start(self) -> ServerProcess {
        let process = self.spawn();
        let line = wait_until(
            BOUND,
            "serving line",
            process.wait_stderr(|line| line.starts_with(SERVING)),
        )
        .await
        .unwrap_or_else(|| panic!("the server exited: {:?}", process.stderr()));
        let socket = served(&line).expect("a serving line").0;
        ServerProcess { socket, ..process }
    }

    /// Runs the server to its exit, for a server that refuses to start. A server that still
    /// runs after [`BOUND`] fails the test.
    pub async fn output(self) -> Output {
        wait_until(BOUND, "exit of the server", self.command().output())
            .await
            .expect("the mu binary runs")
    }

    /// Starts the server and does not wait.
    pub fn spawn(self) -> ServerProcess {
        let mut child = self.command().spawn().expect("the mu binary runs");
        let (lines, _) = watch::channel(Stderr::default());
        let lines = Arc::new(lines);
        let stderr = child.stderr.take().unwrap();
        let sender = Arc::clone(&lines);
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                sender.send_modify(|stderr| stderr.lines.push(line));
            }
            sender.send_modify(|stderr| stderr.closed = true);
        });
        ServerProcess {
            child,
            dir: self.dir,
            socket: PathBuf::new(),
            lines,
        }
    }
}

impl Default for ServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A running `mu server`. The drop kills it.
pub struct ServerProcess {
    child: tokio::process::Child,
    dir: Arc<TempDir>,
    socket: PathBuf,
    lines: Arc<watch::Sender<Stderr>>,
}

impl ServerProcess {
    /// The socket that the server named on stderr.
    pub fn socket(&self) -> &Path {
        &self.socket
    }

    /// The temporary directory of the server.
    pub fn dir(&self) -> &Arc<TempDir> {
        &self.dir
    }

    /// A client of the server.
    pub fn client(&self) -> PortalClient {
        PortalClient::unix(&self.socket)
    }

    /// The lines of stderr so far.
    pub fn stderr(&self) -> Vec<String> {
        self.lines.borrow().lines.clone()
    }

    /// Waits for the first line of stderr that `check` accepts. None means that stderr
    /// closed with no such line.
    pub async fn wait_stderr(&self, check: impl Fn(&str) -> bool) -> Option<String> {
        let mut receiver = self.lines.subscribe();
        let stderr = receiver
            .wait_for(|stderr| stderr.closed || stderr.lines.iter().any(|line| check(line)))
            .await
            .unwrap();
        stderr.lines.iter().find(|line| check(line)).cloned()
    }

    /// Sends `signal` to the server.
    pub fn signal(&self, signal: rustix::process::Signal) {
        let pid = self
            .child
            .id()
            .and_then(|id| rustix::process::Pid::from_raw(id.try_into().unwrap()));
        rustix::process::kill_process(pid.expect("the server runs"), signal).unwrap();
    }

    /// Waits for the exit of the server.
    pub async fn wait_exit(&mut self) -> ExitStatus {
        wait_until(BOUND, "exit of the server", self.child.wait())
            .await
            .unwrap()
    }
}

/// A relay of a Unix socket. It can cut each open connection, and hold, which refuses each
/// new connection until the test releases it.
#[derive(Clone)]
pub struct Relay {
    front: PathBuf,
    live: Arc<Mutex<JoinSet<()>>>,
    held: Arc<watch::Sender<bool>>,
}

impl Relay {
    /// A relay on `front` to `back`.
    pub fn start(front: &Path, back: &Path) -> Self {
        let listener = UnixListener::bind(front).unwrap();
        let relay = Self {
            front: front.to_owned(),
            live: Arc::default(),
            held: Arc::new(watch::channel(false).0),
        };
        let back = back.to_owned();
        let task = relay.clone();
        tokio::spawn(async move {
            while let Ok((mut client, _)) = listener.accept().await {
                if *task.held.borrow() {
                    continue;
                }
                let back = back.clone();
                task.live.lock().unwrap().spawn(async move {
                    let Ok(mut server) = UnixStream::connect(&back).await else {
                        return;
                    };
                    let _ = tokio::io::copy_bidirectional(&mut client, &mut server).await;
                });
            }
        });
        relay
    }

    /// The socket of the relay.
    pub fn socket(&self) -> &Path {
        &self.front
    }

    /// Closes each open connection and returns how many there were.
    pub fn cut(&self) -> usize {
        let mut live = self.live.lock().unwrap();
        let count = live.len();
        live.abort_all();
        count
    }

    /// Refuses each new connection until [`Relay::release`].
    pub fn hold(&self) {
        self.held.send_replace(true);
    }

    /// Lets new connections through again.
    pub fn release(&self) {
        self.held.send_replace(false);
    }
}

/// The one stream of [`mu_on_pty`] that gets the pseudo-terminal.
#[derive(Clone, Copy, Debug)]
pub enum PtyOn {
    /// A terminal on stdin and a pipe on stdout.
    Stdin,
    /// A terminal on stdout and a pipe on stdin.
    Stdout,
}

/// Runs the built binary with a pseudo-terminal on one stream only, and stderr in a pipe.
pub fn mu_on_pty(args: &[&str], on: PtyOn, cwd: &Path) -> Output {
    use rustix::pty::{OpenptFlags, grantpt, openpt, ptsname, unlockpt};

    let master = openpt(OpenptFlags::RDWR | OpenptFlags::NOCTTY).unwrap();
    grantpt(&master).unwrap();
    unlockpt(&master).unwrap();
    let name = ptsname(&master, Vec::new()).unwrap();
    let terminal = File::options()
        .read(true)
        .write(true)
        .open(name.to_str().unwrap())
        .unwrap();
    let mut command = super::mu_command(args, &[]);
    command.current_dir(cwd).stderr(Stdio::piped());
    match on {
        PtyOn::Stdin => command.stdin(terminal).stdout(Stdio::piped()),
        PtyOn::Stdout => command.stdout(terminal).stdin(Stdio::piped()),
    };
    let output = command.output().expect("the mu binary runs");
    drop(master);
    output
}

/// A server with its client and one open event stream, and every event the stream gave.
pub struct Session {
    /// The server.
    pub server: ServerProcess,
    /// A client of the server.
    pub client: PortalClient,
    events: std::pin::Pin<Box<dyn futures::Stream<Item = mu_client::ClientEvent> + Send>>,
    seen: Vec<mu_proto::Event>,
}

impl Session {
    /// Starts the server of `builder` and opens a stream from the first event.
    pub async fn start(builder: ServerBuilder) -> Self {
        let server = builder.start().await;
        let client = server.client();
        let events = Box::pin(client.events());
        Self {
            server,
            client,
            events,
            seen: Vec::new(),
        }
    }

    /// Starts a server with these fake worlds and waits for the greeting of each.
    pub async fn with_worlds(worlds: &[&str]) -> Self {
        let mut session = Self::start(ServerBuilder::new().fake_worlds(worlds)).await;
        for world in worlds {
            session.welcome(world).await;
        }
        session
    }

    /// Reads the stream to the first event, from the start, that `check` accepts.
    pub async fn until(
        &mut self,
        what: &str,
        check: impl Fn(&mu_proto::Event) -> bool,
    ) -> mu_proto::Event {
        if let Some(event) = self.seen.iter().find(|event| check(event)) {
            return event.clone();
        }
        wait_until(BOUND, what, async {
            loop {
                let item = futures::StreamExt::next(&mut self.events).await;
                if let Some(mu_client::ClientEvent::Event(event)) = item {
                    self.seen.push(event.clone());
                    if check(&event) {
                        return event;
                    }
                }
            }
        })
        .await
    }

    /// Reads to the game line `text` of `world`.
    pub async fn line(&mut self, world: &str, text: &str) -> mu_proto::Event {
        self.until(&format!("line {text:?} of {world}"), |event| {
            event.world == world
                && event.kind == mu_proto::EventKind::EVENT_KIND_LINE
                && event.text == text
        })
        .await
    }

    /// Reads to the event of `kind` of `world` after the number `after`.
    pub async fn kind(
        &mut self,
        world: &str,
        kind: mu_proto::EventKind,
        after: u64,
    ) -> mu_proto::Event {
        self.until(&format!("{kind:?} of {world} after {after}"), |event| {
            event.world == world && event.kind == kind && event.seq > after
        })
        .await
    }

    /// Reads to the greeting of `world`, which follows its login.
    pub async fn welcome(&mut self, world: &str) -> mu_proto::Event {
        self.line(world, &greeting(world)).await
    }

    /// Every event that the stream gave so far.
    pub fn seen(&self) -> &[mu_proto::Event] {
        &self.seen
    }

    /// Checks that the stream gives no connect event of `world` after the number `after`
    /// within [`PAST_FIRST_WAIT`].
    pub async fn no_connect_after(&mut self, world: &str, after: u64) {
        let connect = self.kind(world, mu_proto::EventKind::EVENT_KIND_CONNECTED, after);
        let found = tokio::time::timeout(PAST_FIRST_WAIT, connect).await;
        assert!(found.is_err(), "{world} connected again: {:?}", found.ok());
    }
}

/// `mu client` in a pseudo-terminal of 80 by 24, with its screen read by `vt100`.
pub struct PtyClient {
    screen: Arc<watch::Sender<vt100::Parser>>,
    writer: Box<dyn std::io::Write + Send>,
    killer: Box<dyn portable_pty::ChildKiller + Send + Sync>,
    exit: Option<tokio::sync::oneshot::Receiver<portable_pty::ExitStatus>>,
    // The pseudo-terminal must stay open while the client runs.
    _master: Box<dyn portable_pty::MasterPty + Send>,
}

impl PtyClient {
    /// Starts `mu client` with `--socket <socket>` in `cwd`, with [`super::test_env`].
    pub fn start(socket: &Path, cwd: &Path) -> Self {
        use portable_pty::{CommandBuilder, PtySize, native_pty_system};

        let pair = native_pty_system()
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        let mut command = CommandBuilder::new(env!("CARGO_BIN_EXE_mu"));
        command.args(["client", "--socket"]);
        command.arg(socket);
        command.cwd(cwd);
        for (name, value) in super::test_env() {
            match value {
                Some(value) => command.env(name, value),
                None => command.env_remove(name),
            }
        }
        command.env("TERM", "xterm-256color");
        let mut child = pair.slave.spawn_command(command).unwrap();
        drop(pair.slave);
        let killer = child.clone_killer();
        let (exited, exit) = tokio::sync::oneshot::channel();
        std::thread::spawn(move || {
            if let Ok(status) = child.wait() {
                let _ = exited.send(status);
            }
        });
        let screen = Arc::new(
            watch::channel(vt100::Parser::new(
                std::num::NonZeroU16::new(24).unwrap(),
                std::num::NonZeroU16::new(80).unwrap(),
                0,
            ))
            .0,
        );
        let mut reader = pair.master.try_clone_reader().unwrap();
        let sender = Arc::clone(&screen);
        std::thread::spawn(move || {
            let mut buffer = [0_u8; 4096];
            while let Ok(n @ 1..) = reader.read(&mut buffer) {
                sender.send_modify(|parser| parser.process(&buffer[..n]));
            }
        });
        Self {
            screen,
            writer: pair.master.take_writer().unwrap(),
            killer,
            exit: Some(exit),
            _master: pair.master,
        }
    }

    /// The rows of the screen now.
    pub fn rows(&self) -> Vec<String> {
        self.screen.borrow().screen().rows(0, 80).collect()
    }

    /// The text of the screen now.
    pub fn contents(&self) -> String {
        self.screen.borrow().screen().contents()
    }

    /// Waits until the rows pass `check`. A failure prints the screen.
    pub async fn wait_rows(&self, what: &str, check: impl Fn(&[String]) -> bool) {
        let mut receiver = self.screen.subscribe();
        let note = ScreenNote {
            what,
            screen: &self.screen,
        };
        wait_until(BOUND, note, async {
            receiver
                .wait_for(|parser| check(&parser.screen().rows(0, 80).collect::<Vec<_>>()))
                .await
                .is_ok()
        })
        .await;
    }

    /// Waits until a row shows the greeting of `alpha/aria`.
    pub async fn wait_greeting(&self) {
        self.wait_rows("the greeting", |rows| {
            rows.iter().any(|row| row.trim() == greeting("alpha/aria"))
        })
        .await;
    }

    /// Waits until the top row passes `check`.
    pub async fn wait_top(&self, what: &str, check: impl Fn(&str) -> bool) {
        self.wait_rows(what, |rows| rows.first().is_some_and(|top| check(top)))
            .await;
    }

    /// Types `bytes` into the terminal.
    pub fn send(&mut self, bytes: &[u8]) {
        self.writer.write_all(bytes).unwrap();
        self.writer.flush().unwrap();
    }

    /// Waits up to `bound` for the exit of the client. None means that it still runs.
    pub async fn exit_within(&mut self, bound: Duration) -> Option<portable_pty::ExitStatus> {
        let exit = self.exit.as_mut()?;
        match tokio::time::timeout(bound, exit).await {
            Ok(status) => {
                self.exit = None;
                status.ok()
            }
            Err(_elapsed) => None,
        }
    }
}

/// The name of a wait on the screen. It shows the screen as it is when the wait fails.
struct ScreenNote<'a> {
    what: &'a str,
    screen: &'a watch::Sender<vt100::Parser>,
}

impl Display for ScreenNote<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let contents = self.screen.borrow().screen().contents();
        write!(f, "{}; the screen is:\n{contents}", self.what)
    }
}

impl Drop for PtyClient {
    fn drop(&mut self) {
        // A client that exited already is not an error here.
        let _ = self.killer.kill();
    }
}
