use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::java_26_2::configuration::clientbound::packet::{
    ConfigurationClientboundPacket, CustomPayload, RegistryData, RegistryEntry, RegistryTags,
    TagDefinition,
};
use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::value::known_pack::KnownPack;
use crate::java_26_2::value::nbt::TextComponentNbt;

const SYNCHRONIZED_REGISTRIES: [&str; 29] = [
    "minecraft:worldgen/biome",
    "minecraft:chat_type",
    "minecraft:trim_pattern",
    "minecraft:trim_material",
    "minecraft:wolf_variant",
    "minecraft:wolf_sound_variant",
    "minecraft:pig_variant",
    "minecraft:pig_sound_variant",
    "minecraft:frog_variant",
    "minecraft:cat_variant",
    "minecraft:cat_sound_variant",
    "minecraft:cow_sound_variant",
    "minecraft:cow_variant",
    "minecraft:chicken_sound_variant",
    "minecraft:chicken_variant",
    "minecraft:zombie_nautilus_variant",
    "minecraft:painting_variant",
    "minecraft:sulfur_cube_archetype",
    "minecraft:dimension_type",
    "minecraft:damage_type",
    "minecraft:banner_pattern",
    "minecraft:enchantment",
    "minecraft:jukebox_song",
    "minecraft:instrument",
    "minecraft:test_environment",
    "minecraft:test_instance",
    "minecraft:dialog",
    "minecraft:world_clock",
    "minecraft:timeline",
];

const LOCKED_FEATURES: [&str; 4] = [
    "minecraft:vanilla",
    "minecraft:trade_rebalance",
    "minecraft:redstone_experiments",
    "minecraft:minecart_improvements",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationStage {
    AwaitingBrand,
    AwaitingFeatures,
    AwaitingKnownPackOffer,
    AwaitingKnownPackResponse,
    SynchronizingRegistries,
    AwaitingSpawnReadiness,
    AwaitingFinish,
    PlayInstalledAwaitingFinishAcknowledgement,
    Play,
    Disconnected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientAction {
    None,
    EchoKeepAlive(i64),
    EchoPing(i32),
    SelectKnownPacks(Vec<KnownPack>),
    InstallPlayThenAcknowledgeFinish,
    Disconnect(TextComponentNbt),
}

/// A headless-client projection of the connection-local configuration state.
#[derive(Debug, Clone)]
pub struct ConfigurationProjection {
    stage: ConfigurationStage,
    brand: Option<String>,
    enabled_features: BTreeSet<Identifier>,
    offered_packs: Vec<KnownPack>,
    registries: BTreeMap<Identifier, Vec<RegistryEntry>>,
    tags: BTreeMap<Identifier, Vec<TagDefinition>>,
    static_registry_sizes: BTreeMap<Identifier, usize>,
    last_registry_order: Option<usize>,
}

impl ConfigurationProjection {
    #[must_use]
    pub fn new(static_registry_sizes: BTreeMap<Identifier, usize>) -> Self {
        Self {
            stage: ConfigurationStage::AwaitingBrand,
            brand: None,
            enabled_features: BTreeSet::new(),
            offered_packs: Vec::new(),
            registries: BTreeMap::new(),
            tags: BTreeMap::new(),
            static_registry_sizes,
            last_registry_order: None,
        }
    }

    #[must_use]
    pub const fn stage(&self) -> ConfigurationStage {
        self.stage
    }

    #[must_use]
    pub fn brand(&self) -> Option<&str> {
        self.brand.as_deref()
    }

    #[must_use]
    pub fn enabled_features(&self) -> &BTreeSet<Identifier> {
        &self.enabled_features
    }

    #[must_use]
    pub fn offered_packs(&self) -> &[KnownPack] {
        &self.offered_packs
    }

    #[must_use]
    pub fn registry(&self, registry: &Identifier) -> Option<&[RegistryEntry]> {
        self.registries.get(registry).map(Vec::as_slice)
    }

    #[must_use]
    pub fn tags(&self, registry: &Identifier) -> Option<&[TagDefinition]> {
        self.tags.get(registry).map(Vec::as_slice)
    }

    pub fn apply(
        &mut self,
        packet: ConfigurationClientboundPacket,
    ) -> Result<ClientAction, ConfigurationProjectionError> {
        if self.stage == ConfigurationStage::Disconnected {
            return Err(ConfigurationProjectionError::TerminalStage { stage: self.stage });
        }
        match packet {
            ConfigurationClientboundPacket::KeepAlive(token) => {
                Ok(ClientAction::EchoKeepAlive(token))
            }
            ConfigurationClientboundPacket::Ping(token) => Ok(ClientAction::EchoPing(token)),
            ConfigurationClientboundPacket::Disconnect(reason) => {
                self.stage = ConfigurationStage::Disconnected;
                Ok(ClientAction::Disconnect(reason))
            }
            ConfigurationClientboundPacket::CustomPayload(CustomPayload::Brand(brand)) => {
                self.require_stage(ConfigurationStage::AwaitingBrand, "brand")?;
                self.brand = Some(brand);
                self.stage = ConfigurationStage::AwaitingFeatures;
                Ok(ClientAction::None)
            }
            ConfigurationClientboundPacket::CustomPayload(CustomPayload::Discarded { .. }) => {
                Ok(ClientAction::None)
            }
            ConfigurationClientboundPacket::UpdateEnabledFeatures(features) => {
                self.require_stage(ConfigurationStage::AwaitingFeatures, "enabled features")?;
                self.enabled_features = features
                    .into_iter()
                    .filter(|feature| LOCKED_FEATURES.contains(&feature.to_string().as_str()))
                    .collect();
                self.stage = ConfigurationStage::AwaitingKnownPackOffer;
                Ok(ClientAction::None)
            }
            ConfigurationClientboundPacket::SelectKnownPacks(packs) => {
                self.require_stage(
                    ConfigurationStage::AwaitingKnownPackOffer,
                    "known-pack offer",
                )?;
                self.offered_packs.clone_from(&packs);
                self.stage = ConfigurationStage::AwaitingKnownPackResponse;
                Ok(ClientAction::SelectKnownPacks(packs))
            }
            ConfigurationClientboundPacket::RegistryData(data) => {
                self.require_stage(ConfigurationStage::SynchronizingRegistries, "registry data")?;
                self.apply_registry_data(data)?;
                Ok(ClientAction::None)
            }
            ConfigurationClientboundPacket::UpdateTags(registries) => {
                self.require_stage(ConfigurationStage::SynchronizingRegistries, "update tags")?;
                self.apply_tags(registries)?;
                self.stage = ConfigurationStage::AwaitingSpawnReadiness;
                Ok(ClientAction::None)
            }
            ConfigurationClientboundPacket::FinishConfiguration => {
                self.require_stage(ConfigurationStage::AwaitingFinish, "finish configuration")?;
                self.stage = ConfigurationStage::PlayInstalledAwaitingFinishAcknowledgement;
                Ok(ClientAction::InstallPlayThenAcknowledgeFinish)
            }
        }
    }

    pub fn known_pack_response_sent(&mut self) -> Result<(), ConfigurationProjectionError> {
        self.require_stage(
            ConfigurationStage::AwaitingKnownPackResponse,
            "known-pack response",
        )?;
        self.stage = ConfigurationStage::SynchronizingRegistries;
        Ok(())
    }

    pub fn spawn_ready(&mut self) -> Result<(), ConfigurationProjectionError> {
        self.require_stage(
            ConfigurationStage::AwaitingSpawnReadiness,
            "spawn readiness",
        )?;
        self.stage = ConfigurationStage::AwaitingFinish;
        Ok(())
    }

    pub fn finish_acknowledgement_sent(&mut self) -> Result<(), ConfigurationProjectionError> {
        self.require_stage(
            ConfigurationStage::PlayInstalledAwaitingFinishAcknowledgement,
            "finish acknowledgement",
        )?;
        self.stage = ConfigurationStage::Play;
        Ok(())
    }

    fn apply_registry_data(
        &mut self,
        data: RegistryData,
    ) -> Result<(), ConfigurationProjectionError> {
        let order = synchronized_registry_order(&data.registry).ok_or_else(|| {
            ConfigurationProjectionError::UnknownSynchronizedRegistry {
                registry: data.registry.clone(),
            }
        })?;
        if self
            .last_registry_order
            .is_some_and(|previous| order < previous)
        {
            return Err(ConfigurationProjectionError::RegistryOrder {
                registry: data.registry,
            });
        }
        let entries = self.registries.entry(data.registry.clone()).or_default();
        let mut ids: BTreeSet<Identifier> = entries.iter().map(|entry| entry.id.clone()).collect();
        for entry in data.entries {
            if !ids.insert(entry.id.clone()) {
                return Err(ConfigurationProjectionError::DuplicateRegistryEntry {
                    registry: data.registry,
                    entry: entry.id,
                });
            }
            entries.push(entry);
        }
        self.last_registry_order = Some(order);
        Ok(())
    }

    fn apply_tags(
        &mut self,
        registries: Vec<RegistryTags>,
    ) -> Result<(), ConfigurationProjectionError> {
        for payload in registries {
            let size = self
                .registries
                .get(&payload.registry)
                .map(Vec::len)
                .or_else(|| self.static_registry_sizes.get(&payload.registry).copied());
            let mut seen = BTreeSet::new();
            for tag in &payload.tags {
                if !seen.insert(tag.id.clone()) {
                    return Err(ConfigurationProjectionError::DuplicateTag {
                        registry: payload.registry,
                        tag: tag.id.clone(),
                    });
                }
                for member in &tag.members {
                    let index = usize::try_from(*member).ok();
                    if size.is_some_and(|size| index.is_none_or(|index| index >= size))
                        || size.is_none() && index.is_none()
                    {
                        return Err(ConfigurationProjectionError::TagMemberOutOfRange {
                            registry: payload.registry,
                            tag: tag.id.clone(),
                            member: *member,
                            size,
                        });
                    }
                }
            }
            self.tags.insert(payload.registry, payload.tags);
        }
        Ok(())
    }

    fn require_stage(
        &self,
        expected: ConfigurationStage,
        packet: &'static str,
    ) -> Result<(), ConfigurationProjectionError> {
        if self.stage == expected {
            Ok(())
        } else {
            Err(ConfigurationProjectionError::UnexpectedStage {
                packet,
                expected,
                actual: self.stage,
            })
        }
    }
}

impl Default for ConfigurationProjection {
    fn default() -> Self {
        Self::new(BTreeMap::new())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConfigurationProjectionError {
    #[error("{packet} requires stage {expected:?}, but connection is {actual:?}")]
    UnexpectedStage {
        packet: &'static str,
        expected: ConfigurationStage,
        actual: ConfigurationStage,
    },
    #[error("configuration projection is terminal in stage {stage:?}")]
    TerminalStage { stage: ConfigurationStage },
    #[error("registry {registry} is not in the locked synchronized-registry sequence")]
    UnknownSynchronizedRegistry { registry: Identifier },
    #[error("registry {registry} arrived out of locked synchronization order")]
    RegistryOrder { registry: Identifier },
    #[error("registry {registry} contains duplicate entry {entry}")]
    DuplicateRegistryEntry {
        registry: Identifier,
        entry: Identifier,
    },
    #[error("registry {registry} contains duplicate tag {tag}")]
    DuplicateTag {
        registry: Identifier,
        tag: Identifier,
    },
    #[error(
        "tag {tag} in registry {registry} references member {member} outside registry size {size:?}"
    )]
    TagMemberOutOfRange {
        registry: Identifier,
        tag: Identifier,
        member: i32,
        size: Option<usize>,
    },
}

fn synchronized_registry_order(registry: &Identifier) -> Option<usize> {
    let value = registry.to_string();
    SYNCHRONIZED_REGISTRIES
        .iter()
        .position(|candidate| *candidate == value)
}
