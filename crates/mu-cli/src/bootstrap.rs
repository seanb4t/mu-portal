//! The values that a client needs before a server answers: the URL of the server, with the
//! longest wait for an answer, and the name of the device.

use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

use clap::Args;
use mu_config::DeviceName;
use url::Url;

use crate::client::{DEFAULT_WAIT, connect_wait};
use crate::socket::SocketArgs;

/// The variable that names the server URL.
pub const SERVER_ENV: &str = "MU_SERVER";
/// The variable that names the device.
pub const DEVICE_ENV: &str = "MU_DEVICE";

/// The query option of the longest wait for an answer.
const CONNECT_WAIT: &str = "connect_wait";

/// The flags of the server URL and of the device name.
#[derive(Clone, Debug, Default, Args)]
pub struct BootstrapArgs {
    #[arg(
        long,
        env = SERVER_ENV,
        value_name = "URL",
        help = "The URL of the server: unix:// and the path of its socket. The query option \
                connect_wait sets the seconds to wait for an answer"
    )]
    server: Option<String>,
    /// The name of this device. The default is the host name in lowercase
    #[arg(long, value_name = "NAME")]
    device: Option<String>,
}

/// The server that a client calls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Server {
    /// The URL as the messages name it.
    pub url: String,
    /// The path of the socket.
    pub socket: PathBuf,
    /// The longest wait for an answer.
    pub wait: Duration,
}

impl BootstrapArgs {
    /// The server of `--server`, then of `MU_SERVER`, then the default socket. An empty value
    /// counts as unset.
    ///
    /// # Errors
    ///
    /// The text that names the URL and the reason.
    pub fn server(&self) -> Result<Server, String> {
        if let Some(url) = self.server.as_deref().filter(|url| !url.is_empty()) {
            return parse_url(url);
        }
        let socket = SocketArgs::default().resolve();
        Ok(Server {
            url: format!("unix://{}", socket.display()),
            socket,
            wait: DEFAULT_WAIT,
        })
    }

    /// The device of `--device`, then of `MU_DEVICE`, then the host name in lowercase.
    ///
    /// # Errors
    ///
    /// The text that names the bad name.
    pub fn device(&self) -> Result<DeviceName, String> {
        device_name(
            self.device.as_deref(),
            std::env::var_os(DEVICE_ENV),
            host_name,
        )
    }
}

/// The host name of the machine in lowercase.
fn host_name() -> String {
    rustix::system::uname()
        .nodename()
        .to_string_lossy()
        .to_lowercase()
}

/// The device of `flag`, then of the variable `var`, then of `host`. An empty variable counts
/// as unset, and an empty flag is a bad name.
///
/// # Errors
///
/// The text that names the bad name. A bad host name also names `--device`.
pub fn device_name(
    flag: Option<&str>,
    var: Option<OsString>,
    host: impl FnOnce() -> String,
) -> Result<DeviceName, String> {
    if let Some(name) = flag {
        return DeviceName::new(name).map_err(|error| error.to_string());
    }
    if let Some(name) = var.filter(|name| !name.is_empty()) {
        let name = name.to_string_lossy();
        return DeviceName::new(&name).map_err(|error| format!("{DEVICE_ENV}: {error}"));
    }
    let host = host();
    DeviceName::new(&host).map_err(|error| {
        format!(
            "the host name {host:?} is not a device name, so set --device or {DEVICE_ENV}: {}",
            error.reason
        )
    })
}

/// The server of `text`: `unix://`, an absolute path with percent escapes, and the option
/// `connect_wait`.
///
/// # Errors
///
/// The text that names the URL and the reason.
pub fn parse_url(text: &str) -> Result<Server, String> {
    let refused = |reason: &str| format!("{text}: {reason}");
    let url = Url::parse(text).map_err(|error| refused(&error.to_string()))?;
    if url.scheme() != "unix" {
        return Err(refused(&format!(
            "the scheme is {}: a server URL is unix:// and the path of a socket",
            url.scheme()
        )));
    }
    if !url.has_authority() {
        return Err(refused("write unix:// and an absolute path"));
    }
    if url.host_str().is_some_and(|host| !host.is_empty()) {
        return Err(refused("a server URL has no host"));
    }
    if url.fragment().is_some() {
        return Err(refused("a server URL has no fragment"));
    }
    let socket = url
        .to_file_path()
        .map_err(|()| refused("the path is not absolute"))?;
    let mut wait = None;
    for (name, value) in url.query_pairs() {
        if name != CONNECT_WAIT {
            return Err(refused(&format!(
                "{name} is no option: the only option is {CONNECT_WAIT}"
            )));
        }
        if wait.is_some() {
            return Err(refused(&format!("{CONNECT_WAIT} is there twice")));
        }
        let seconds: f64 = value
            .parse()
            .map_err(|_| refused(&format!("{CONNECT_WAIT}: {value:?} is not a number")))?;
        wait = Some(
            connect_wait(seconds)
                .map_err(|reason| refused(&format!("{CONNECT_WAIT}: {reason}")))?,
        );
    }
    Ok(Server {
        url: text.to_owned(),
        socket,
        wait: wait.unwrap_or(DEFAULT_WAIT),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bad_host_name_names_the_device_flag() {
        let error = device_name(None, None, || "my mac".to_owned()).unwrap_err();
        assert!(
            error.contains("\"my mac\"") && error.contains("--device"),
            "{error}"
        );
    }

    #[test]
    fn a_decoded_path() {
        assert_eq!(
            parse_url("unix:///tmp/a%20b/s.sock").unwrap().socket,
            PathBuf::from("/tmp/a b/s.sock")
        );
    }
}
