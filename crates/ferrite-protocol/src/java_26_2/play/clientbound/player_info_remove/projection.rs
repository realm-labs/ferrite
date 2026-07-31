use std::collections::{BTreeMap, BTreeSet};

use crate::java_26_2::play::clientbound::player_info_remove::PlayerInfoRemove;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedPlayerInfo {
    pub profile_name: String,
    pub listed: bool,
    pub chat_session: Option<u128>,
    pub object_token: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SocialRelationship {
    pub hidden: bool,
    pub blocked: bool,
    pub friend: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerInfoRemovalEffect {
    pub profile_id: u128,
    pub removed_object_token: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayerInfoRemovalProjection {
    entries: BTreeMap<u128, ProjectedPlayerInfo>,
    listed_objects: BTreeSet<u64>,
    discovered_names: BTreeMap<String, u128>,
    relationships: BTreeMap<u128, SocialRelationship>,
    social_removals: Vec<u128>,
}

impl PlayerInfoRemovalProjection {
    pub fn install(&mut self, profile_id: u128, info: ProjectedPlayerInfo) {
        if let Some(previous) = self.entries.insert(profile_id, info.clone()) {
            self.listed_objects.remove(&previous.object_token);
        }
        if info.listed {
            self.listed_objects.insert(info.object_token);
        }
        self.discovered_names
            .insert(info.profile_name.clone(), profile_id);
    }

    pub fn set_relationship(&mut self, profile_id: u128, relationship: SocialRelationship) {
        self.relationships.insert(profile_id, relationship);
    }

    pub fn apply(&mut self, packet: &PlayerInfoRemove) -> Vec<PlayerInfoRemovalEffect> {
        packet
            .profile_ids
            .iter()
            .map(|profile_id| {
                self.social_removals.push(*profile_id);
                let removed_object_token = self.entries.remove(profile_id).map(|entry| {
                    if entry.listed {
                        self.listed_objects.remove(&entry.object_token);
                    }
                    entry.object_token
                });
                PlayerInfoRemovalEffect {
                    profile_id: *profile_id,
                    removed_object_token,
                }
            })
            .collect()
    }

    #[must_use]
    pub fn entries(&self) -> &BTreeMap<u128, ProjectedPlayerInfo> {
        &self.entries
    }

    #[must_use]
    pub fn listed_objects(&self) -> &BTreeSet<u64> {
        &self.listed_objects
    }

    #[must_use]
    pub fn discovered_names(&self) -> &BTreeMap<String, u128> {
        &self.discovered_names
    }

    #[must_use]
    pub fn relationships(&self) -> &BTreeMap<u128, SocialRelationship> {
        &self.relationships
    }

    #[must_use]
    pub fn social_removals(&self) -> &[u128] {
        &self.social_removals
    }

    #[must_use]
    pub fn online_names(&self) -> BTreeSet<String> {
        self.entries
            .values()
            .map(|entry| entry.profile_name.clone())
            .collect()
    }

    #[must_use]
    pub fn chat_session(&self, profile_id: u128) -> Option<u128> {
        self.entries
            .get(&profile_id)
            .and_then(|entry| entry.chat_session)
    }
}
