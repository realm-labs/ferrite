//! Mate compatibility, approach timing, and generic/special child commits.

#[must_use]
pub const fn base_can_mate(
    self_candidate: bool,
    same_runtime_class: bool,
    actor_love: u16,
    partner_love: u16,
) -> bool {
    !self_candidate && same_runtime_class && actor_love > 0 && partner_love > 0
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MateCandidate {
    pub distance_squared: f64,
    pub can_mate: bool,
    pub panicking: bool,
}

#[must_use]
pub fn nearest_mate(candidates_in_query_order: &[MateCandidate]) -> Option<usize> {
    let mut selected = None;
    let mut distance = f64::INFINITY;
    for (index, candidate) in candidates_in_query_order.iter().enumerate() {
        if candidate.can_mate && !candidate.panicking && candidate.distance_squared < distance {
            selected = Some(index);
            distance = candidate.distance_squared;
        }
    }
    selected
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BreedGoalTick {
    pub look_at_partner: bool,
    pub request_navigation: bool,
    pub love_time: u16,
    pub attempt_breeding: bool,
}

#[must_use]
pub const fn breed_goal_tick(love_time: u16, distance_squared: f64) -> BreedGoalTick {
    let love_time = love_time.saturating_add(1);
    BreedGoalTick {
        look_at_partner: true,
        request_navigation: true,
        love_time,
        attempt_breeding: love_time >= 30 && distance_squared < 9.0,
    }
}

#[must_use]
pub const fn breed_goal_continues(
    partner_alive: bool,
    partner_in_love: bool,
    partner_panicking: bool,
    love_time: u16,
) -> bool {
    partner_alive && partner_in_love && !partner_panicking && love_time < 60
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildCommit {
    NullRetryWithoutChanges,
    Generic(GenericChildCommit),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenericChildCommit {
    pub set_child_baby: bool,
    pub snap_to_actor_zero_rotation: bool,
    pub cause_actor_then_partner: bool,
    pub award_and_trigger_before_parent_cooldowns: bool,
    pub parent_age: u16,
    pub clear_both_love: bool,
    pub broadcast_actor_event: u8,
    pub xp: Option<u8>,
    pub xp_before_child_insertion: bool,
    pub insertion_result_ignored: bool,
}

#[must_use]
pub const fn generic_child_commit(
    child_constructed: bool,
    mob_drops: bool,
    xp_draw: u8,
) -> ChildCommit {
    if !child_constructed {
        ChildCommit::NullRetryWithoutChanges
    } else {
        ChildCommit::Generic(GenericChildCommit {
            set_child_baby: true,
            snap_to_actor_zero_rotation: true,
            cause_actor_then_partner: true,
            award_and_trigger_before_parent_cooldowns: true,
            parent_age: 6_000,
            clear_both_love: true,
            broadcast_actor_event: 18,
            xp: if mob_drops {
                Some(1 + xp_draw % 7)
            } else {
                None
            },
            xp_before_child_insertion: true,
            insertion_result_ignored: true,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialBreeding {
    Fox,
    Turtle,
    Frog,
    Sniffer,
    Allay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpecialOrder {
    pub generic_finalization: bool,
    pub child_or_item_before_event: bool,
    pub event_before_xp: bool,
    pub parent_cooldown: u16,
}

#[must_use]
pub const fn special_order(kind: SpecialBreeding) -> SpecialOrder {
    match kind {
        SpecialBreeding::Fox => SpecialOrder {
            generic_finalization: false,
            child_or_item_before_event: true,
            event_before_xp: true,
            parent_cooldown: 6_000,
        },
        SpecialBreeding::Turtle | SpecialBreeding::Frog | SpecialBreeding::Sniffer => {
            SpecialOrder {
                generic_finalization: true,
                child_or_item_before_event: false,
                event_before_xp: false,
                parent_cooldown: 6_000,
            }
        }
        SpecialBreeding::Allay => SpecialOrder {
            generic_finalization: false,
            child_or_item_before_event: true,
            event_before_xp: true,
            parent_cooldown: 6_000,
        },
    }
}
