//! One call to a running server for a verb that never starts one, such as `mu config` and
//! `mu layout`.

use std::fs;

use mu_client::PortalClient;

use crate::bootstrap::Server;
use crate::client::answered;
use crate::socket::{checked_dir, current_uid, socket_dir};

/// Runs `call` with the client of `server` on a new runtime. The directory of the socket must
/// be private to this user.
///
/// # Errors
///
/// The text of the error of the directory check, of the runtime, or of the call.
pub fn call<T, F: Future<Output = Result<T, mu_client::ClientError>>>(
    server: &Server,
    call: impl FnOnce(PortalClient) -> F,
) -> Result<T, String> {
    let dir = socket_dir(&server.socket);
    if fs::symlink_metadata(dir).is_ok() {
        checked_dir(dir, current_uid()).map_err(|error| error.to_string())?;
    }
    let runtime = tokio::runtime::Runtime::new().map_err(|error| error.to_string())?;
    runtime.block_on(async {
        let client = PortalClient::unix(&server.socket);
        answered(&server.url, server.wait, call(client)).await
    })
}
