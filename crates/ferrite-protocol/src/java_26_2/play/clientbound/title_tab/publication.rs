use std::collections::BTreeMap;

use crate::java_26_2::play::clientbound::title_tab::packet::{
    ClearTitles, SelectAdvancementsTab, SetActionBarText, SetSubtitleText, SetTitleText,
    SetTitlesAnimation,
};
use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::value::nbt::TextComponentNbt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleDelivery<T> {
    pub recipient: u128,
    pub packet: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPublication<T, E> {
    pub deliveries: Vec<TitleDelivery<T>>,
    pub failure: Option<E>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedTitleKind {
    ActionBar,
    Subtitle,
    Title,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedTitlePacket {
    ActionBar(SetActionBarText),
    Subtitle(SetSubtitleText),
    Title(SetTitleText),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanonicalTitleTimes(SetTitlesAnimation);

impl CanonicalTitleTimes {
    #[must_use]
    pub const fn new(fade_in: i32, stay: i32, fade_out: i32) -> Option<Self> {
        if fade_in < 0 || stay < 0 || fade_out < 0 {
            None
        } else {
            Some(Self(SetTitlesAnimation {
                fade_in,
                stay,
                fade_out,
            }))
        }
    }

    #[must_use]
    pub const fn packet(self) -> SetTitlesAnimation {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementTabDefinition {
    pub id: Identifier,
    pub root: Identifier,
    pub has_display: bool,
}

#[derive(Debug, Clone, Default)]
pub struct AdvancementTabPublisher {
    definitions: BTreeMap<Identifier, AdvancementTabDefinition>,
    selected: Option<Identifier>,
}

impl AdvancementTabPublisher {
    #[must_use]
    pub fn new(definitions: impl IntoIterator<Item = AdvancementTabDefinition>) -> Self {
        Self {
            definitions: definitions
                .into_iter()
                .map(|definition| (definition.id.clone(), definition))
                .collect(),
            selected: None,
        }
    }

    pub fn select(&mut self, requested: Option<&Identifier>) -> Option<SelectAdvancementsTab> {
        let selected = requested
            .and_then(|requested| self.definitions.get(requested))
            .filter(|definition| definition.id == definition.root && definition.has_display)
            .map(|definition| definition.id.clone());
        if self.selected == selected {
            None
        } else {
            self.selected = selected.clone();
            Some(SelectAdvancementsTab { tab: selected })
        }
    }

    #[must_use]
    pub fn selected(&self) -> Option<&Identifier> {
        self.selected.as_ref()
    }

    pub fn reload(&mut self, definitions: impl IntoIterator<Item = AdvancementTabDefinition>) {
        self.definitions = definitions
            .into_iter()
            .map(|definition| (definition.id.clone(), definition))
            .collect();
    }
}

#[must_use]
pub fn publish_clear(
    selected_players: &[u128],
    reset_times: bool,
) -> Vec<TitleDelivery<ClearTitles>> {
    publish_shared(selected_players, ClearTitles { reset_times })
}

#[must_use]
pub fn publish_animation(
    selected_players: &[u128],
    times: CanonicalTitleTimes,
) -> Vec<TitleDelivery<SetTitlesAnimation>> {
    publish_shared(selected_players, times.packet())
}

pub fn publish_resolved<E>(
    selected_players: &[u128],
    kind: ResolvedTitleKind,
    mut resolve: impl FnMut(u128) -> Result<TextComponentNbt, E>,
) -> ResolvedPublication<ResolvedTitlePacket, E> {
    let mut deliveries = Vec::with_capacity(selected_players.len());
    for recipient in selected_players {
        let text = match resolve(*recipient) {
            Ok(text) => text,
            Err(error) => {
                return ResolvedPublication {
                    deliveries,
                    failure: Some(error),
                };
            }
        };
        let packet = match kind {
            ResolvedTitleKind::ActionBar => {
                ResolvedTitlePacket::ActionBar(SetActionBarText { text })
            }
            ResolvedTitleKind::Subtitle => ResolvedTitlePacket::Subtitle(SetSubtitleText { text }),
            ResolvedTitleKind::Title => ResolvedTitlePacket::Title(SetTitleText { text }),
        };
        deliveries.push(TitleDelivery {
            recipient: *recipient,
            packet,
        });
    }
    ResolvedPublication {
        deliveries,
        failure: None,
    }
}

fn publish_shared<T: Clone>(selected_players: &[u128], packet: T) -> Vec<TitleDelivery<T>> {
    selected_players
        .iter()
        .map(|recipient| TitleDelivery {
            recipient: *recipient,
            packet: packet.clone(),
        })
        .collect()
}
