//! XP eligibility, reward mutation, greedy splitting, and orb merge selection.

use crate::entity::runtime::ent_007::drops::EquipmentSlot;

pub const XP_SPLIT_THRESHOLDS: [u32; 11] = [2477, 1237, 617, 307, 149, 73, 37, 17, 7, 3, 1];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExperienceOwner {
    Player,
    CommonLiving,
    Monster,
    Tadpole,
}

#[must_use]
pub const fn experience_eligible(input: ExperienceEligibility) -> bool {
    if input.skip_drop_experience {
        return false;
    }
    if matches!(input.owner, ExperienceOwner::Player) {
        return true;
    }
    input.recent_player_memory
        && input.should_drop_experience
        && input.mob_drops
        && match input.owner {
            ExperienceOwner::CommonLiving => input.adult,
            ExperienceOwner::Monster => true,
            ExperienceOwner::Tadpole => false,
            ExperienceOwner::Player => true,
        }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExperienceEligibility {
    pub owner: ExperienceOwner,
    pub skip_drop_experience: bool,
    pub recent_player_memory: bool,
    pub should_drop_experience: bool,
    pub mob_drops: bool,
    pub adult: bool,
}

#[must_use]
pub const fn player_experience_reward(level: u32, keep_inventory: bool, spectator: bool) -> u32 {
    if keep_inventory || spectator {
        0
    } else {
        let reward = level.saturating_mul(7);
        if reward > 100 { 100 } else { reward }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EquipmentXpInput {
    pub slot: EquipmentSlot,
    pub nonempty: bool,
    pub drop_chance: f32,
    pub draw_three: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MobExperience {
    pub reward: u32,
    pub draws_consumed: usize,
}

#[must_use]
pub fn mob_experience_reward(
    base_reward: u32,
    inputs_in_slot_order: &[EquipmentXpInput],
    hoglin_or_piglin: bool,
) -> MobExperience {
    if base_reward == 0 || hoglin_or_piglin {
        return MobExperience {
            reward: base_reward,
            draws_consumed: 0,
        };
    }
    let mut reward = base_reward;
    let mut draws = 0;
    for input in inputs_in_slot_order {
        if !matches!(input.slot, EquipmentSlot::Saddle)
            && input.nonempty
            && input.drop_chance <= 1.0
        {
            reward += 1 + u32::from(input.draw_three % 3);
            draws += 1;
        }
    }
    MobExperience {
        reward,
        draws_consumed: draws,
    }
}

#[must_use]
pub const fn next_xp_split(remaining: u32) -> u32 {
    let mut index = 0;
    while index < XP_SPLIT_THRESHOLDS.len() {
        let threshold = XP_SPLIT_THRESHOLDS[index];
        if remaining >= threshold {
            return threshold;
        }
        index += 1;
    }
    0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrbCandidate {
    pub id: i32,
    pub value: u32,
    pub removed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrbAward {
    Merge {
        candidate_index: usize,
        increment_count: bool,
        reset_age: bool,
    },
    Spawn {
        exact_position: bool,
        zero_requested_direction: bool,
    },
    None,
}

#[must_use]
pub fn award_orb(
    piece: u32,
    random_offset_forty: i32,
    candidates_in_iteration_order: &[OrbCandidate],
) -> OrbAward {
    if piece == 0 {
        return OrbAward::None;
    }
    for (index, candidate) in candidates_in_iteration_order.iter().enumerate() {
        if !candidate.removed
            && candidate.value == piece
            && (candidate.id - random_offset_forty).rem_euclid(40) == 0
        {
            return OrbAward::Merge {
                candidate_index: index,
                increment_count: true,
                reset_age: true,
            };
        }
    }
    OrbAward::Spawn {
        exact_position: true,
        zero_requested_direction: true,
    }
}
