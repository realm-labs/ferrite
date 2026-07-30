use std::collections::BTreeMap;

use crate::java_26_2::play::serverbound::inventory_auxiliary::packet::SeenAdvancements;
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementNode {
    pub id: Identifier,
    pub root: Identifier,
    pub has_display: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementTabCorrection {
    pub selected: Option<Identifier>,
}

#[derive(Debug, Clone, Default)]
pub struct AdvancementTabState {
    nodes: BTreeMap<Identifier, AdvancementNode>,
    selected: Option<Identifier>,
}

impl AdvancementTabState {
    #[must_use]
    pub fn new(nodes: impl IntoIterator<Item = AdvancementNode>) -> Self {
        Self {
            nodes: nodes
                .into_iter()
                .map(|node| (node.id.clone(), node))
                .collect(),
            selected: None,
        }
    }

    #[must_use]
    pub fn selected(&self) -> Option<&Identifier> {
        self.selected.as_ref()
    }

    pub fn handle(&mut self, packet: &SeenAdvancements) -> Option<AdvancementTabCorrection> {
        let SeenAdvancements::OpenedTab(requested) = packet else {
            return None;
        };
        let node = self.nodes.get(requested)?;
        let selected = (node.id == node.root && node.has_display).then(|| node.id.clone());
        if self.selected == selected {
            None
        } else {
            self.selected = selected.clone();
            Some(AdvancementTabCorrection { selected })
        }
    }

    pub fn reload(&mut self, nodes: impl IntoIterator<Item = AdvancementNode>) {
        self.nodes = nodes
            .into_iter()
            .map(|node| (node.id.clone(), node))
            .collect();
        self.selected = None;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdvancementClientEvent {
    Sent(SeenAdvancements),
    Notified(Option<Identifier>),
}

#[derive(Debug, Clone, Default)]
pub struct AdvancementClientScreen {
    connected: bool,
    selected: Option<Identifier>,
    pub events: Vec<AdvancementClientEvent>,
}

impl AdvancementClientScreen {
    #[must_use]
    pub fn connected() -> Self {
        Self {
            connected: true,
            ..Self::default()
        }
    }

    pub fn open_tab(&mut self, selected: Identifier) -> SeenAdvancements {
        let packet = SeenAdvancements::OpenedTab(selected.clone());
        self.events
            .push(AdvancementClientEvent::Sent(packet.clone()));
        if self.selected.as_ref() != Some(&selected) {
            self.selected = Some(selected.clone());
            self.events
                .push(AdvancementClientEvent::Notified(Some(selected)));
        }
        packet
    }

    pub fn remove(&mut self) -> Option<SeenAdvancements> {
        if !self.connected {
            return None;
        }
        let packet = SeenAdvancements::ClosedScreen;
        self.events
            .push(AdvancementClientEvent::Sent(packet.clone()));
        Some(packet)
    }

    pub fn apply_correction(&mut self, correction: AdvancementTabCorrection) {
        if self.selected != correction.selected {
            self.selected = correction.selected.clone();
            self.events
                .push(AdvancementClientEvent::Notified(correction.selected));
        }
    }

    pub fn disconnect(&mut self) {
        self.connected = false;
    }

    #[must_use]
    pub fn selected(&self) -> Option<&Identifier> {
        self.selected.as_ref()
    }
}
