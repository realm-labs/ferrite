use ferrite_region_runtime::topology::layout::TopologyLayout;
use serde::{Deserialize, Serialize};
use std::error::Error;

const CAPACITY_PROFILES: &str = include_str!("../../../../benchmarks/capacity-profiles.toml");

#[derive(Debug, Deserialize)]
struct ProfileDocument {
    schema_version: u16,
    profile: Vec<CapacityProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct CapacityProfile {
    pub name: String,
    pub regions: u16,
    pub nodes: u16,
    pub worlds: u16,
    pub mailbox_capacity: usize,
    pub warmup_ticks: u64,
    pub measured_ticks: u64,
    pub samples: u16,
    pub hotspot_percent: u8,
    pub rebalance_max_skew_regions: u16,
}

pub(super) fn load() -> Result<Vec<CapacityProfile>, Box<dyn Error>> {
    let document: ProfileDocument = toml::from_str(CAPACITY_PROFILES)?;
    if document.schema_version != 1 {
        return Err(format!(
            "capacity profile schema {} is unsupported",
            document.schema_version
        )
        .into());
    }
    if document.profile.is_empty() {
        return Err("at least one capacity profile is required".into());
    }
    let mut names = std::collections::BTreeSet::new();
    for profile in &document.profile {
        validate(profile)?;
        if !names.insert(profile.name.as_str()) {
            return Err(format!("capacity profile {} is duplicated", profile.name).into());
        }
    }
    Ok(document.profile)
}

pub(super) fn select(name: Option<&str>) -> Result<Vec<CapacityProfile>, Box<dyn Error>> {
    let profiles = load()?;
    match name {
        None | Some("all") => Ok(profiles),
        Some(name) => profiles
            .into_iter()
            .find(|profile| profile.name == name)
            .map(|profile| vec![profile])
            .ok_or_else(|| format!("unknown capacity profile {name}").into()),
    }
}

fn validate(profile: &CapacityProfile) -> Result<(), Box<dyn Error>> {
    if profile.name.is_empty()
        || !profile
            .name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(format!("capacity profile name {:?} is not canonical", profile.name).into());
    }
    if profile.regions < 2 || profile.nodes < 2 || profile.nodes > profile.regions {
        return Err(format!("capacity profile {} has invalid topology", profile.name).into());
    }
    if profile.worlds == 0 || profile.regions / profile.worlds < 2 {
        return Err(format!("capacity profile {} has invalid world count", profile.name).into());
    }
    if profile.mailbox_capacity < usize::from(profile.regions)
        || profile.warmup_ticks == 0
        || profile.measured_ticks == 0
        || profile.samples < 3
    {
        return Err(format!("capacity profile {} has invalid bounds", profile.name).into());
    }
    if !(51..=95).contains(&profile.hotspot_percent) || profile.rebalance_max_skew_regions != 1 {
        return Err(format!("capacity profile {} has invalid objectives", profile.name).into());
    }
    Ok(())
}

pub(super) fn balanced_layout(profile: &CapacityProfile) -> Result<TopologyLayout, Box<dyn Error>> {
    Ok(TopologyLayout::multiverse_ring(
        profile.regions,
        profile.nodes,
        profile.worlds,
    )?)
}

pub(super) fn hotspot_layout(profile: &CapacityProfile) -> Result<TopologyLayout, Box<dyn Error>> {
    let balanced = balanced_layout(profile)?;
    let hotspot_regions = usize::from(profile.regions)
        .saturating_mul(usize::from(profile.hotspot_percent))
        .div_ceil(100);
    let descriptors = balanced
        .descriptors()
        .cloned()
        .enumerate()
        .map(|(index, mut descriptor)| {
            descriptor.node = if index < hotspot_regions {
                0
            } else {
                1 + u16::try_from(index - hotspot_regions)
                    .expect("profile Region count is bounded by u16")
                    % (profile.nodes - 1)
            };
            descriptor
        })
        .collect();
    Ok(TopologyLayout::new(descriptors, profile.nodes)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn committed_profiles_are_valid_and_cover_many_worlds() {
        let profiles = load().unwrap();
        assert_eq!(profiles.len(), 3);
        for profile in profiles {
            let balanced = balanced_layout(&profile).unwrap();
            let worlds = balanced
                .descriptors()
                .map(|descriptor| descriptor.key.world())
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(worlds.len(), usize::from(profile.worlds));
            assert_eq!(balanced.len(), usize::from(profile.regions));
            assert_eq!(
                hotspot_layout(&profile).unwrap().node_count(),
                profile.nodes
            );
        }
    }

    #[test]
    fn unknown_profile_is_rejected() {
        assert!(select(Some("missing-profile")).is_err());
    }
}
