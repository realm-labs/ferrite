use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::value::nbt::{NetworkNbt, TextComponentNbt};

/// One required clientbound packet legal in the 26.2 configuration state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigurationClientboundPacket {
    CustomPayload(CustomPayload),
    Disconnect(TextComponentNbt),
    FinishConfiguration,
    KeepAlive(i64),
    Ping(i32),
    RegistryData(RegistryData),
    UpdateEnabledFeatures(BTreeSet<Identifier>),
    UpdateTags(Vec<RegistryTags>),
    SelectKnownPacks(Vec<KnownPack>),
}

/// Configuration custom payloads understood by the vanilla client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomPayload {
    Brand(String),
    /// Vanilla consumes but does not retain an unknown channel's payload.
    Discarded {
        channel: Identifier,
        length: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryData {
    pub registry: Identifier,
    pub entries: Vec<RegistryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryEntry {
    pub id: Identifier,
    pub data: Option<NetworkNbt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryTags {
    pub registry: Identifier,
    pub tags: Vec<TagDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagDefinition {
    pub id: Identifier,
    pub members: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownPack {
    pub namespace: String,
    pub id: String,
    pub version: String,
}

impl KnownPack {
    #[must_use]
    pub fn vanilla_core() -> Self {
        Self {
            namespace: "minecraft".to_owned(),
            id: "core".to_owned(),
            version: "26.2".to_owned(),
        }
    }
}
use std::collections::BTreeSet;
