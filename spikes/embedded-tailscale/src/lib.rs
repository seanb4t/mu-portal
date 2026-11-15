//! The interface that hides the Tailscale library from the server.
//!
//! Only this file names the `tailscale` crate. The server sees `JoinConfig`,
//! `Node`, `Identity` and an axum listener.
//! A tailnet peer is named by `Node::peer_name`.

use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;

/// What the server needs to join a tailnet.
pub struct JoinConfig {
    /// The directory that holds the node keys. The same directory gives the
    /// same node after a restart.
    pub state_dir: PathBuf,
    pub hostname: String,
    /// Needed only for the first join. Later starts use the stored keys.
    pub auth_key: Option<String>,
}

/// The facts that tell whether a restart kept the node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    pub node_key: String,
    pub stable_id: String,
    pub ipv4: Ipv4Addr,
    pub name: String,
}

#[derive(Debug)]
pub enum JoinError {
    /// The control server needs a browser login, or a new auth key.
    NeedsLogin(String),
    Library(String),
}

impl std::fmt::Display for JoinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JoinError::NeedsLogin(url) => write!(f, "the node is not authorized; log in at {url}"),
            JoinError::Library(e) => write!(f, "tailscale: {e}"),
        }
    }
}

impl std::error::Error for JoinError {}

fn lib_err(e: impl std::fmt::Display) -> JoinError {
    JoinError::Library(e.to_string())
}

/// The tailnet address of a caller, for `axum::extract::ConnectInfo`.
#[derive(Debug, Clone, Copy)]
pub struct Caller(pub SocketAddr);

impl axum::extract::connect_info::Connected<axum::serve::IncomingStream<'_, Listener>> for Caller {
    fn connect_info(stream: axum::serve::IncomingStream<'_, Listener>) -> Self {
        Caller(*stream.remote_addr())
    }
}

/// Makes the state directory private to the owner before the library writes
/// private keys into it. The library writes the key file with the process
/// umask, so the directory mode is the only guard until the file is restricted.
pub fn prepare_state_dir(dir: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    restrict(dir, 0o700)
}

fn restrict(path: &std::path::Path, mode: u32) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
}

/// A node on the tailnet.
pub struct Node {
    device: tailscale::Device,
}

/// An axum listener on the tailnet.
pub type Listener = tailscale::axum::Listener;

impl Node {
    pub async fn join(config: JoinConfig) -> Result<Node, JoinError> {
        let key_file = config.state_dir.join("keys.json");
        prepare_state_dir(&config.state_dir).map_err(lib_err)?;
        let mut ts = tailscale::Config::default_with_key_file(&key_file)
            .await
            .map_err(lib_err)?;
        restrict(&key_file, 0o600).map_err(lib_err)?;
        ts.requested_hostname = Some(config.hostname);
        ts.client_name = Some("mu-portal".into());
        let device = tailscale::Device::new(&ts, config.auth_key)
            .await
            .map_err(lib_err)?;
        match device.is_authorized().await.map_err(lib_err)? {
            tailscale::AuthState::Authorized => Ok(Node { device }),
            tailscale::AuthState::NotAuthorized(url) => Err(JoinError::NeedsLogin(url.to_string())),
        }
    }

    pub async fn identity(&self) -> Result<Identity, JoinError> {
        let me = self.device.self_node().await.map_err(lib_err)?;
        Ok(Identity {
            node_key: format!("{:?}", me.node_key),
            stable_id: format!("{:?}", me.stable_id),
            ipv4: self.device.ipv4_addr().await.map_err(lib_err)?,
            name: me.hostname,
        })
    }

    /// The name of the tailnet peer at `ip`, if the tailnet knows it.
    pub async fn peer_name(&self, ip: std::net::IpAddr) -> Result<Option<String>, JoinError> {
        let peer = self.device.peer_by_tailnet_ip(ip).await.map_err(lib_err)?;
        Ok(peer.map(|p| p.hostname))
    }

    pub async fn listen(&self, port: u16) -> Result<Listener, JoinError> {
        let ip = self.device.ipv4_addr().await.map_err(lib_err)?;
        let listener = self
            .device
            .tcp_listen(SocketAddr::from((ip, port)))
            .await
            .map_err(lib_err)?;
        Ok(listener.into())
    }

    /// Reports whether the node stopped before the timeout.
    pub async fn leave(self, timeout: Duration) -> bool {
        self.device.shutdown(Some(timeout)).await
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn state_dir_is_private_to_the_owner() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("state");
        std::fs::create_dir(&dir).unwrap();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();

        super::prepare_state_dir(&dir).unwrap();

        let mode = std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o700);
    }
}
