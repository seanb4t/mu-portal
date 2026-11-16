//! Helpers that run the built binary as a server, relay its socket, or give it a
//! pseudo-terminal on one stream.

use std::ffi::OsString;
use std::fmt::Display;
use std::fs::File;
use std::future::Future;
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

/// The start of the line that the server writes when it serves.
pub const SERVING: &str = "mu server: serving the API on ";

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

/// The arguments and the environment of a `mu server` to start.
pub struct ServerBuilder {
    dir: Arc<TempDir>,
    socket_flag: Option<PathBuf>,
    args: Vec<OsString>,
    env: Vec<(OsString, Option<OsString>)>,
}

impl ServerBuilder {
    /// A server with `--socket <dir>/s.sock` in a new temporary directory, with no
    /// `MU_SOCKET`.
    pub fn new() -> Self {
        let dir = Arc::new(tempfile::tempdir().unwrap());
        let socket_flag = Some(dir.path().join("s.sock"));
        Self {
            dir,
            socket_flag,
            args: Vec::new(),
            env: vec![("MU_SOCKET".into(), None)],
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
        for (name, value) in &self.env {
            match value {
                Some(value) => command.env(name, value),
                None => command.env_remove(name),
            };
        }
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
        .await;
        let socket = PathBuf::from(line.trim_start_matches(SERVING));
        ServerProcess { socket, ..process }
    }

    /// Starts the server and does not wait.
    pub fn spawn(self) -> ServerProcess {
        let mut child = self.command().spawn().expect("the mu binary runs");
        let (lines, _) = watch::channel(Vec::new());
        let lines = Arc::new(lines);
        let stderr = child.stderr.take().unwrap();
        let sender = Arc::clone(&lines);
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                sender.send_modify(|lines| lines.push(line));
            }
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
    lines: Arc<watch::Sender<Vec<String>>>,
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
        self.lines.borrow().clone()
    }

    /// Waits for the first line of stderr that `check` accepts.
    pub async fn wait_stderr(&self, check: impl Fn(&str) -> bool) -> String {
        let mut receiver = self.lines.subscribe();
        let lines = receiver
            .wait_for(|lines| lines.iter().any(|line| check(line)))
            .await
            .unwrap();
        lines.iter().find(|line| check(line)).unwrap().clone()
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
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_mu"));
    command.args(args).current_dir(cwd).stderr(Stdio::piped());
    match on {
        PtyOn::Stdin => command.stdin(terminal).stdout(Stdio::piped()),
        PtyOn::Stdout => command.stdout(terminal).stdin(Stdio::piped()),
    };
    let output = command.output().expect("the mu binary runs");
    drop(master);
    output
}
