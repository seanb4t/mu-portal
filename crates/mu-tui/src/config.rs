//! The client configuration that the server sends, and how the model applies it.

use std::collections::BTreeMap;

use mu_client::{ClientError, PortalClient};
pub use mu_config::Echo;
use mu_config::layout::Layout;
use mu_config::{Kind, Label, Os, Segment};
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
    /// - The layout, the layout of each world entry, the label and the segments take their
    ///   values if they pass the checks of `mu-config`. Else the old value stays, and a notice
    ///   names the field.
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
        self.apply_layouts(&reply);
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

    /// Takes the layouts, the label and the segments of `reply`. A reply with no line of a
    /// layout gives the default layout.
    fn apply_layouts(&mut self, reply: &GetClientConfigResponse) {
        if !reply.layout.is_empty() {
            match Layout::parse(&reply.layout) {
                Ok(layout) => {
                    self.layout = layout;
                    self.layout_name.clone_from(&reply.layout_name);
                }
                Err(error) => self.notice(&format!(
                    "the server sent the bad layout {}: {error}",
                    reply.layout_name
                )),
            }
        }
        let mut worlds = BTreeMap::new();
        for entry in &reply.world_layouts {
            match Layout::parse(&entry.lines) {
                Ok(layout) => {
                    worlds.insert(entry.key.clone(), (entry.name.clone(), layout));
                }
                Err(error) => self.notice(&format!(
                    "the server sent the bad layout {} for {}: {error}",
                    entry.name, entry.key
                )),
            }
        }
        self.world_layouts = worlds;
        if !reply.tab_label.is_empty() {
            match Label::parse(&reply.tab_label) {
                Ok(label) => self.label = label,
                Err(error) => self.notice(&format!("the server sent the bad tab label: {error}")),
            }
        }
        match (
            Segment::check(&reply.statusline_left),
            Segment::check(&reply.statusline_right),
        ) {
            (Ok(left), Ok(right)) => self.segments = (left, right),
            (Err(error), _) | (_, Err(error)) => {
                self.notice(&format!("the server sent a bad statusline: {error}"));
            }
        }
    }
}
