use std::collections::BTreeSet;

use thiserror::Error;

use crate::java_26_2::configuration::clientbound::packet::{
    ConfigurationClientboundPacket, CustomPayload, RegistryData, RegistryEntry, RegistryTags,
};
use crate::java_26_2::configuration::registry::SYNCHRONIZED_REGISTRY_IDENTITIES;
use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::value::known_pack::KnownPack;
use crate::java_26_2::value::nbt::NetworkNbt;

const LOCKED_FEATURES: [&str; 4] = [
    "minecraft:vanilla",
    "minecraft:trade_rebalance",
    "minecraft:redstone_experiments",
    "minecraft:minecart_improvements",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryProjection {
    pub registry: Identifier,
    pub entries: Vec<RegistryProjectionEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryProjectionEntry {
    pub id: Identifier,
    pub data: Option<NetworkNbt>,
    pub source_pack: Option<KnownPack>,
}

/// Immutable, validated data used by every connection during required configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigurationSnapshot {
    brand: String,
    enabled_features: BTreeSet<Identifier>,
    offered_packs: Vec<KnownPack>,
    registries: Vec<RegistryProjection>,
    tags: Vec<RegistryTags>,
}

impl ConfigurationSnapshot {
    pub fn new(
        brand: String,
        enabled_features: BTreeSet<Identifier>,
        offered_packs: Vec<KnownPack>,
        registries: Vec<RegistryProjection>,
        tags: Vec<RegistryTags>,
    ) -> Result<Self, ConfigurationSnapshotError> {
        validate_features(&enabled_features)?;
        validate_registries(&registries, &offered_packs)?;
        validate_tags(&registries, &tags)?;
        Ok(Self {
            brand,
            enabled_features,
            offered_packs,
            registries,
            tags,
        })
    }

    #[must_use]
    pub fn offered_packs(&self) -> &[KnownPack] {
        &self.offered_packs
    }

    #[must_use]
    pub fn initial_packets(&self) -> [ConfigurationClientboundPacket; 3] {
        [
            ConfigurationClientboundPacket::CustomPayload(CustomPayload::Brand(self.brand.clone())),
            ConfigurationClientboundPacket::UpdateEnabledFeatures(self.enabled_features.clone()),
            ConfigurationClientboundPacket::SelectKnownPacks(self.offered_packs.clone()),
        ]
    }

    #[must_use]
    pub fn synchronization_packets(
        &self,
        exact_offer_match: bool,
    ) -> Vec<ConfigurationClientboundPacket> {
        let mut packets = Vec::with_capacity(self.registries.len() + 1);
        for registry in &self.registries {
            let entries = registry
                .entries
                .iter()
                .map(|entry| RegistryEntry {
                    id: entry.id.clone(),
                    data: if exact_offer_match
                        && entry
                            .source_pack
                            .as_ref()
                            .is_some_and(|pack| self.offered_packs.contains(pack))
                    {
                        None
                    } else {
                        entry.data.clone()
                    },
                })
                .collect();
            packets.push(ConfigurationClientboundPacket::RegistryData(RegistryData {
                registry: registry.registry.clone(),
                entries,
            }));
        }
        packets.push(ConfigurationClientboundPacket::UpdateTags(
            self.tags.clone(),
        ));
        packets
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConfigurationSnapshotError {
    #[error("required configuration must enable minecraft:vanilla")]
    MissingVanillaFeature,
    #[error("feature {feature} is outside the locked 26.2 feature set")]
    UnsupportedFeature { feature: Identifier },
    #[error("required configuration has {actual} synchronized registries instead of {expected}")]
    RegistryCount { expected: usize, actual: usize },
    #[error("synchronized registry index {index} is {actual}, expected {expected}")]
    RegistryOrder {
        index: usize,
        expected: &'static str,
        actual: Identifier,
    },
    #[error("registry {registry} repeats element {entry}")]
    DuplicateRegistryEntry {
        registry: Identifier,
        entry: Identifier,
    },
    #[error("registry element {entry} names source pack {pack:?}, which is not offered")]
    UnofferedSourcePack { entry: Identifier, pack: KnownPack },
    #[error("tag payload repeats registry {registry}")]
    DuplicateTagRegistry { registry: Identifier },
    #[error("registry {registry} repeats tag {tag}")]
    DuplicateTag {
        registry: Identifier,
        tag: Identifier,
    },
    #[error(
        "tag {tag} in registry {registry} references member {member} outside dynamic size {size}"
    )]
    TagMemberOutOfRange {
        registry: Identifier,
        tag: Identifier,
        member: i32,
        size: usize,
    },
    #[error("tag {tag} in registry {registry} contains negative member {member}")]
    NegativeTagMember {
        registry: Identifier,
        tag: Identifier,
        member: i32,
    },
}

fn validate_features(
    enabled_features: &BTreeSet<Identifier>,
) -> Result<(), ConfigurationSnapshotError> {
    if !enabled_features
        .iter()
        .any(|feature| feature.to_string() == "minecraft:vanilla")
    {
        return Err(ConfigurationSnapshotError::MissingVanillaFeature);
    }
    for feature in enabled_features {
        if !LOCKED_FEATURES.contains(&feature.to_string().as_str()) {
            return Err(ConfigurationSnapshotError::UnsupportedFeature {
                feature: feature.clone(),
            });
        }
    }
    Ok(())
}

fn validate_registries(
    registries: &[RegistryProjection],
    offered_packs: &[KnownPack],
) -> Result<(), ConfigurationSnapshotError> {
    if registries.len() != SYNCHRONIZED_REGISTRY_IDENTITIES.len() {
        return Err(ConfigurationSnapshotError::RegistryCount {
            expected: SYNCHRONIZED_REGISTRY_IDENTITIES.len(),
            actual: registries.len(),
        });
    }
    for (index, (registry, expected)) in registries
        .iter()
        .zip(SYNCHRONIZED_REGISTRY_IDENTITIES)
        .enumerate()
    {
        if registry.registry.to_string() != expected {
            return Err(ConfigurationSnapshotError::RegistryOrder {
                index,
                expected,
                actual: registry.registry.clone(),
            });
        }
        let mut entries = BTreeSet::new();
        for entry in &registry.entries {
            if !entries.insert(entry.id.clone()) {
                return Err(ConfigurationSnapshotError::DuplicateRegistryEntry {
                    registry: registry.registry.clone(),
                    entry: entry.id.clone(),
                });
            }
            if let Some(pack) = &entry.source_pack
                && !offered_packs.contains(pack)
            {
                return Err(ConfigurationSnapshotError::UnofferedSourcePack {
                    entry: entry.id.clone(),
                    pack: pack.clone(),
                });
            }
        }
    }
    Ok(())
}

fn validate_tags(
    registries: &[RegistryProjection],
    tags: &[RegistryTags],
) -> Result<(), ConfigurationSnapshotError> {
    let mut tag_registries = BTreeSet::new();
    for payload in tags {
        if !tag_registries.insert(payload.registry.clone()) {
            return Err(ConfigurationSnapshotError::DuplicateTagRegistry {
                registry: payload.registry.clone(),
            });
        }
        let dynamic_size = registries
            .iter()
            .find(|registry| registry.registry == payload.registry)
            .map(|registry| registry.entries.len());
        let mut tag_ids = BTreeSet::new();
        for tag in &payload.tags {
            if !tag_ids.insert(tag.id.clone()) {
                return Err(ConfigurationSnapshotError::DuplicateTag {
                    registry: payload.registry.clone(),
                    tag: tag.id.clone(),
                });
            }
            for member in &tag.members {
                let Ok(index) = usize::try_from(*member) else {
                    return Err(ConfigurationSnapshotError::NegativeTagMember {
                        registry: payload.registry.clone(),
                        tag: tag.id.clone(),
                        member: *member,
                    });
                };
                if dynamic_size.is_some_and(|size| index >= size) {
                    return Err(ConfigurationSnapshotError::TagMemberOutOfRange {
                        registry: payload.registry.clone(),
                        tag: tag.id.clone(),
                        member: *member,
                        size: dynamic_size.unwrap_or_default(),
                    });
                }
            }
        }
    }
    Ok(())
}
