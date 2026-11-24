//! The client configuration that the server sends, and how the model applies it.

use std::collections::BTreeMap;

use mu_client::{ClientError, PortalClient};
pub use mu_config::Echo;
use mu_config::{Kind, Os};
use mu_proto::GetClientConfigResponse;

use crate::keymap::Keymap;
use crate::model::Model;

/// The client configuration of the terminal client on this machine with the name `device`.
///
/// # Errors
///
/// The [`ClientError`] of the call.
pub async fn fetch(
    client: &PortalClient,
    device: &str,
) -> Result<GetClientConfigResponse, ClientError> {
    client
        .client_config(Kind::Tui.name(), Os::current().name(), device)
        .await
}

impl Model {
    /// Applies the client configuration `reply`, which becomes the last applied reply with its
    /// store hash. A reply with the revision of the last applied reply changes nothing else. For
    /// a reply with another revision:
    ///
    /// - The keymap takes its bindings if they pass [`mu_config::check_keys`]. Else the keymap
    ///   stays, and a notice names the bad binding.
    /// - The echo takes its echo if [`Echo::named`] knows the name. Else the echo stays, and a
    ///   notice names it.
    /// - Each of its notices that the last applied reply did not hold becomes a notice line.
    pub fn apply_config(&mut self, reply: GetClientConfigResponse) {
        let last = self.config.take();
        if let Some(last) = last.as_ref()
            && last.revision == reply.revision
        {
            self.config = Some(reply);
            return;
        }
        let keys: BTreeMap<String, String> = reply
            .bindings
            .iter()
            .map(|binding| (binding.key.clone(), binding.entry.clone()))
            .collect();
        match mu_config::check_keys(&keys) {
            Ok(bindings) => self.keymap = Keymap::with_bindings(&bindings),
            Err(error) => self.notice(&format!(
                "the server sent the bad binding {}: {error}",
                error.key
            )),
        }
        match Echo::named(&reply.echo) {
            Some(echo) => self.echo = echo,
            None => self.notice(&format!("the server sent the bad echo {:?}", reply.echo)),
        }
        for notice in &reply.notices {
            let shown = last
                .as_ref()
                .is_some_and(|last| last.notices.iter().any(|each| each.text == notice.text));
            if !shown {
                self.notice(&notice.text);
            }
        }
        self.config = Some(reply);
    }
}
