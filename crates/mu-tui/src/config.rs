//! The client configuration that the server sends, and how the model applies it.

use std::collections::BTreeMap;

pub use mu_config::Echo;
use mu_proto::GetClientConfigResponse;

use crate::keymap::Keymap;
use crate::model::Model;

impl Model {
    /// Applies the client configuration `reply`. A reply with a new revision replaces the keymap
    /// and the echo, and each of its notices that the last applied reply did not hold becomes a
    /// notice line. A reply with the revision of the last applied reply changes nothing.
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
