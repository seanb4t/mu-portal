//! The changes of the layout in the client: each one changes the plan on the screen at once,
//! and returns the effect that saves it on the server.

use mu_config::layout::{Block, BlockId, Change, Layout, LineRef, REWRITTEN, Step};
use mu_config::{Choice, LayoutName, SaveTarget, WorldKey, save_target};

use crate::model::Model;
use crate::update::Effect;

impl Model {
    /// The key of the entry that gives the layout on the screen, or none for the field `layout`.
    fn choice_key(&self) -> Option<String> {
        match self.choice() {
            Choice::Entry(key) => Some(key.as_str().to_owned()),
            Choice::Default => None,
        }
    }

    /// Puts `layout` on the screen under the name `name`, in place of the layout of the choice
    /// in use.
    fn set_chosen(&mut self, name: String, layout: Layout) {
        if let Some(key) = self.choice_key() {
            self.world_layouts.insert(key, (name, layout));
        } else {
            self.layout_name = name;
            self.layout = layout;
        }
    }

    /// Puts `layout` on the screen under the new name `name`, and returns the effect that
    /// saves it. The choice in use then names it.
    fn save_as(&mut self, name: &LayoutName, layout: Layout) -> Vec<Effect> {
        let lines = layout.lines();
        let then = Some(self.choice());
        self.set_chosen(name.as_str().to_owned(), layout);
        vec![Effect::SaveLayout {
            name: name.as_str().to_owned(),
            lines,
            device: self.device.clone(),
            then,
        }]
    }

    /// Puts `layout` on the screen and saves nothing: a step of a drag, or of arrange mode.
    pub(crate) fn show_layout(&mut self, layout: Layout) {
        let name = self.layout_name().to_owned();
        self.set_chosen(name, layout);
    }

    /// The effect that saves the layout on the screen. A player layout keeps its name, and each
    /// world that uses it gets the change. A built-in layout gives a new layout, and the choice
    /// in use then names it.
    pub(crate) fn save_shown(&mut self) -> Vec<Effect> {
        let name = self.layout_name().to_owned();
        let layout = self.layout().clone();
        let lines = layout.lines();
        match save_target(&self.choice(), &name, Some(&self.device)) {
            Ok(SaveTarget::Same(name)) => {
                if self.layout_name == name.as_str() {
                    self.layout = layout.clone();
                }
                for (held, plan) in self.world_layouts.values_mut() {
                    if held.as_str() == name.as_str() {
                        *plan = layout.clone();
                    }
                }
                vec![Effect::SaveLayout {
                    name: name.as_str().to_owned(),
                    lines,
                    device: self.device.clone(),
                    then: None,
                }]
            }
            Ok(SaveTarget::New(target)) => self.save_as(&target, layout),
            Err(error) => {
                self.notice(&format!(
                    "the layout is not saved: {error}. Use /layout save <name>"
                ));
                Vec::new()
            }
        }
    }

    /// Applies `change` to the layout on the screen and saves the result. A change that the
    /// layout rules refuse changes nothing and gives a notice.
    pub(crate) fn change_layout(&mut self, change: Change) -> Vec<Effect> {
        match self.layout().apply(change) {
            // A change that gives the same plan changes nothing, so it saves nothing.
            Ok(changed) if &changed.layout == self.layout() => Vec::new(),
            Ok(changed) => {
                // The notice goes to an output of the new layout, so it shows after the change.
                self.show_layout(changed.layout);
                if changed.rewritten {
                    self.notice(REWRITTEN);
                }
                self.save_shown()
            }
            Err(error) => {
                self.notice(&error.to_string());
                Vec::new()
            }
        }
    }

    /// The change of `/place <sentence>`.
    pub(crate) fn place(&mut self, sentence: &str) -> Vec<Effect> {
        match Change::place(sentence) {
            Ok(change) => self.change_layout(change),
            Err(error) => {
                self.notice(&format!("{sentence}: {error}"));
                Vec::new()
            }
        }
    }

    /// The change of `/pane hide <id>`.
    pub(crate) fn pane_hide(&mut self, id: &str) -> Vec<Effect> {
        match BlockId::new(id) {
            Ok(id) => self.change_layout(Change::Hide(id)),
            Err(reason) => {
                self.notice(&format!("{id}: {reason}"));
                Vec::new()
            }
        }
    }

    /// The change of `/pane size <id> <size>`. The size `fill` removes the size.
    pub(crate) fn pane_size(&mut self, text: &str) -> Vec<Effect> {
        let Some((id, size)) = text.split_once(' ') else {
            self.notice("/pane size needs the id of a block and a size");
            return Vec::new();
        };
        let change = BlockId::new(id)
            .map_err(str::to_owned)
            .and_then(|id| Change::sized(id, size).map_err(|error| error.to_string()));
        match change {
            Ok(change) => self.change_layout(change),
            Err(reason) => {
                self.notice(&format!("{text}: {reason}"));
                Vec::new()
            }
        }
    }

    /// Shows the next block of the first stack of the screen, in the order of the tree.
    pub(crate) fn pane_next(&mut self) {
        let mut stacks: Vec<Vec<Block>> = Vec::new();
        collect_stacks(self.layout().tree(), &mut stacks);
        let Some(stack) = stacks.first() else {
            self.hint("the layout has no stack");
            return;
        };
        let first = stack[0].id.clone();
        let front = self.front_of(stack).map_or(&first, |block| &block.id);
        let place = stack
            .iter()
            .position(|block| &block.id == front)
            .unwrap_or(0);
        let next = stack[(place + 1) % stack.len()].id.clone();
        self.fronts.insert(first, next);
    }

    /// The hints of `/layout plan`: each line with its number.
    pub(crate) fn layout_plan(&mut self) {
        for line in self.layout().numbered() {
            self.hint(&line);
        }
    }

    /// The change of `/layout insert <number> <sentence>`.
    pub(crate) fn layout_insert(&mut self, text: &str) -> Vec<Effect> {
        let parsed = text.split_once(' ').and_then(|(number, sentence)| {
            Some((number.parse::<usize>().ok()?, Step::parse(sentence.trim())))
        });
        match parsed {
            Some((number, Ok(step))) => self.change_layout(Change::Insert(number, step)),
            Some((_, Err(error))) => {
                self.notice(&format!("{text}: {error}"));
                Vec::new()
            }
            None => {
                self.notice("/layout insert needs the number of the new line and a sentence");
                Vec::new()
            }
        }
    }

    /// The change of `/layout delete <line>`.
    pub(crate) fn layout_delete(&mut self, line: &str) -> Vec<Effect> {
        match LineRef::parse(line.trim()) {
            Ok(line) => self.change_layout(Change::Delete(line)),
            Err(reason) => {
                self.notice(&format!("{line}: {reason}"));
                Vec::new()
            }
        }
    }

    /// The change of `/layout edit <line> <sentence>`.
    pub(crate) fn layout_edit(&mut self, text: &str) -> Vec<Effect> {
        let Some((line, sentence)) = text.split_once(' ') else {
            self.notice("/layout edit needs a line and a sentence");
            return Vec::new();
        };
        let change = LineRef::parse(line)
            .map_err(str::to_owned)
            .and_then(|line| {
                Step::parse(sentence.trim())
                    .map(|step| Change::Replace(line, step))
                    .map_err(|error| error.to_string())
            });
        match change {
            Ok(change) => self.change_layout(change),
            Err(reason) => {
                self.notice(&format!("{text}: {reason}"));
                Vec::new()
            }
        }
    }

    /// The effect of `/layout use <name>`: the field `layout` of the device layer takes `name`.
    pub(crate) fn layout_use(&mut self, name: &str) -> Vec<Effect> {
        if self.choice() != Choice::Default {
            self.notice(
                "this world has a layout of its own, so the screen does not change. Use /layout here <name>",
            );
        }
        vec![Effect::UseLayout {
            device: self.device.clone(),
            name: Some(name.to_owned()),
            choice: Choice::Default,
        }]
    }

    /// The effect of `/layout here <name>`, or of `/layout here` with no name.
    pub(crate) fn layout_here(&mut self, name: Option<&str>) -> Vec<Effect> {
        let Some(world) = self.focused().map(str::to_owned) else {
            self.notice("no world has the focus, so /layout here sets no layout");
            return Vec::new();
        };
        let Ok(key) = world.parse::<WorldKey>() else {
            self.notice(&format!(
                "{world} is no world id, so /layout here sets no layout"
            ));
            return Vec::new();
        };
        vec![Effect::UseLayout {
            device: self.device.clone(),
            name: name.map(str::to_owned),
            choice: Choice::Entry(key),
        }]
    }

    /// The effect of `/layout save <name>`: the layout on the screen under `name`, which the
    /// choice in use then names.
    pub(crate) fn layout_save(&mut self, name: &str) -> Vec<Effect> {
        match LayoutName::new(name) {
            Ok(name) => {
                let layout = self.layout().clone();
                self.save_as(&name, layout)
            }
            Err(error) => {
                self.notice(&error.to_string());
                Vec::new()
            }
        }
    }

    /// The effect of the next layout after the one on the screen, in the order of `names`.
    pub(crate) fn layout_after(&mut self, names: &[String]) -> Vec<Effect> {
        if names.is_empty() {
            return Vec::new();
        }
        let now = self.layout_name();
        let place = names.iter().position(|name| name == now);
        let next = place.map_or(0, |place| (place + 1) % names.len());
        vec![Effect::UseLayout {
            device: self.device.clone(),
            name: Some(names[next].clone()),
            choice: self.choice(),
        }]
    }
}

/// Each pane of `node` with more than one block, in the order of the tree.
fn collect_stacks(node: &mu_config::layout::Node, stacks: &mut Vec<Vec<Block>>) {
    match node {
        mu_config::layout::Node::Pane(blocks) if blocks.len() > 1 => stacks.push(blocks.clone()),
        mu_config::layout::Node::Pane(_) => {}
        mu_config::layout::Node::Split { parts, .. } => {
            for part in parts {
                collect_stacks(&part.node, stacks);
            }
        }
    }
}
