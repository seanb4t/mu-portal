//! The client configuration that the server sends, and how the model applies it.

use std::collections::BTreeMap;

use mu_client::{ClientError, PortalClient};
pub use mu_config::Echo;
use mu_config::layout::{DEFAULT_LAYOUT, Layout};
use mu_config::{Kind, Label, Os, Segment};
use mu_proto::GetClientConfigResponse;

use crate::keymap::Keymap;
use crate::model::{Model, Served};

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

/// An edit of the layout that ended with no save.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Ended {
    /// No edit was on.
    Nothing,
    /// A drag of a border that had moved.
    Drag,
    /// Arrange mode.
    Arrange,
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
    /// - The layout, the label and the segments take their values if they pass the checks of
    ///   `mu-config`. Else the old value stays, and a notice names the field. The layouts of
    ///   the world entries replace the old ones, and a bad one gives a notice and no entry.
    /// - Arrange mode, a drag of a border and a press on a title row end, because their layouts
    ///   are of the old reply.
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
        self.end_edits("the configuration changed", false);
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

    /// Ends arrange mode, a drag of a border and a press on a title row with no save, and says
    /// that `why` ended them. Their layouts belong to the choice and the configuration of
    /// their start. With `put_back`, the layout from before the edit goes back on the screen,
    /// under the choice in use: a caller that replaces the layouts itself passes `false`.
    pub(crate) fn end_edits(&mut self, why: &str, put_back: bool) {
        let ended = self.take_edits(put_back);
        self.say_ended(why, ended);
    }

    /// Ends the edits as [`Self::end_edits`] does, and says nothing yet. The result is what
    /// ended, for [`Self::say_ended`].
    pub(crate) fn take_edits(&mut self, put_back: bool) -> Ended {
        let drag = self.drag.take().filter(|drag| drag.moved);
        let mode = self.arrange.take();
        if put_back {
            let before = mode
                .as_ref()
                .map(|mode| mode.before.clone())
                .or_else(|| drag.as_ref().map(|drag| drag.from.clone()));
            if let Some(before) = before {
                self.show_layout(before);
            }
        }
        match (mode, drag) {
            (Some(_), _) => Ended::Arrange,
            (None, Some(_)) => Ended::Drag,
            (None, None) => Ended::Nothing,
        }
    }

    /// Says that `why` ended the edit `ended`.
    pub(crate) fn say_ended(&mut self, why: &str, ended: Ended) {
        match ended {
            Ended::Arrange => self.notice(&format!("{why}, so arrange mode ended with no save")),
            Ended::Drag => self.hint(&format!("{why}, so the drag ended with no save")),
            Ended::Nothing => {}
        }
    }

    /// Puts back the layouts that the last configuration gave, after a change that the server
    /// did not take. With no configuration yet, that is the default layout and no layout of a
    /// world.
    pub(crate) fn restore_layouts(&mut self) {
        self.end_edits("the server did not take the change", false);
        if let Some(served) = self.served.clone() {
            self.layout_name = served.name;
            self.layout = served.layout;
            self.world_layouts = served.worlds;
        } else {
            self.layout = crate::model::default_layout();
            DEFAULT_LAYOUT.clone_into(&mut self.layout_name);
            self.world_layouts.clear();
        }
        self.show_unshown();
    }

    /// Takes the layouts, the label and the segments of `reply`. A reply with no line of a
    /// layout keeps the layout.
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
        self.served = Some(Served {
            name: self.layout_name.clone(),
            layout: self.layout.clone(),
            worlds: self.world_layouts.clone(),
        });
        // A layout with an `output` can now show the lines that waited for one.
        self.show_unshown();
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
