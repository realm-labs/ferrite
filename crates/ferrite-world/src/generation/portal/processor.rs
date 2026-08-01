//! Per-entity portal timers, cooldown, eligibility, and client confusion state.

use ferrite_foundation::coordinate::BlockPos;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortalTransitionKind {
    None,
    Confusion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortalProcessor {
    pub portal_object: u32,
    pub entry_block: BlockPos,
    pub accumulated_time: i32,
    pub inside_this_tick: bool,
}

impl PortalProcessor {
    pub const fn new(portal_object: u32, entry_block: BlockPos) -> Self {
        Self {
            portal_object,
            entry_block,
            accumulated_time: 0,
            inside_this_tick: true,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PortalContactState {
    pub processor: Option<PortalProcessor>,
    pub cooldown: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContactResult {
    ProcessorCreated,
    ProcessorReplaced,
    ProcessorMarked,
    CooldownRefreshed,
}

impl PortalContactState {
    pub fn contact(
        &mut self,
        portal_object: u32,
        entry_block: BlockPos,
        full_cooldown: i32,
    ) -> ContactResult {
        if self.cooldown > 0 {
            self.cooldown = full_cooldown.max(0);
            return ContactResult::CooldownRefreshed;
        }
        match self.processor.as_mut() {
            None => {
                self.processor = Some(PortalProcessor::new(portal_object, entry_block));
                ContactResult::ProcessorCreated
            }
            Some(processor) if processor.portal_object != portal_object => {
                self.processor = Some(PortalProcessor::new(portal_object, entry_block));
                ContactResult::ProcessorReplaced
            }
            Some(processor) => {
                if !processor.inside_this_tick {
                    processor.entry_block = entry_block;
                }
                processor.inside_this_tick = true;
                ContactResult::ProcessorMarked
            }
        }
    }

    /// Decrements cooldown first, then evaluates the portal processor.
    pub fn tick(&mut self, eligible: bool, wait: i32) -> PortalTickResult {
        if self.cooldown > 0 {
            self.cooldown -= 1;
        }
        let Some(processor) = self.processor.as_mut() else {
            return PortalTickResult::Idle;
        };
        if processor.inside_this_tick {
            processor.inside_this_tick = false;
            if !eligible {
                return PortalTickResult::Ineligible;
            }
            let old_time = processor.accumulated_time;
            processor.accumulated_time = processor.accumulated_time.saturating_add(1);
            if old_time >= wait.max(0) {
                return PortalTickResult::Ready {
                    portal_object: processor.portal_object,
                    entry_block: processor.entry_block,
                };
            }
            PortalTickResult::Accumulating {
                old_time,
                new_time: processor.accumulated_time,
            }
        } else {
            processor.accumulated_time = processor.accumulated_time.saturating_sub(4).max(0);
            if processor.accumulated_time == 0 {
                self.processor = None;
                PortalTickResult::Expired
            } else {
                PortalTickResult::Decayed(processor.accumulated_time)
            }
        }
    }

    /// Starts cooldown before the destination resolver is called.
    pub fn attempt_ready<T>(
        &mut self,
        full_cooldown: i32,
        resolve: impl FnOnce() -> Option<T>,
    ) -> Option<T> {
        self.cooldown = full_cooldown.max(0);
        resolve()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortalTickResult {
    Idle,
    Ineligible,
    Accumulating {
        old_time: i32,
        new_time: i32,
    },
    Decayed(i32),
    Expired,
    Ready {
        portal_object: u32,
        entry_block: BlockPos,
    },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PortalEntityEligibility {
    pub alive: bool,
    pub passenger: bool,
    pub passenger_permitted: bool,
    pub sleeping_living: bool,
    pub fishing_hook: bool,
    pub wither: bool,
    pub ender_dragon: bool,
    pub heart_bound_creaking: bool,
    pub throwable_projectile: bool,
}

pub const fn can_use_portal(entity: PortalEntityEligibility) -> bool {
    if entity.throwable_projectile {
        return true;
    }
    entity.alive
        && (!entity.passenger || entity.passenger_permitted)
        && !entity.sleeping_living
        && !entity.fishing_hook
        && !entity.wither
        && !entity.ender_dragon
        && !entity.heart_bound_creaking
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PortalWaitInput {
    pub is_player: bool,
    pub invulnerable_ability: bool,
    pub creative_delay: i32,
    pub default_delay: i32,
}

pub const fn nether_portal_wait(input: PortalWaitInput) -> i32 {
    if !input.is_player {
        0
    } else if input.invulnerable_ability {
        if input.creative_delay < 0 {
            0
        } else {
            input.creative_delay
        }
    } else if input.default_delay < 0 {
        0
    } else {
        input.default_delay
    }
}

pub const fn entity_portal_cooldown(
    is_server_player: bool,
    first_passenger_is_server_player: bool,
) -> i32 {
    if is_server_player || first_passenger_is_server_player {
        10
    } else {
        300
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ConfusionState {
    pub intensity: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ConfusionTick {
    pub close_disallowed_screen: bool,
    pub close_open_container: bool,
    pub play_trigger_sound: bool,
    pub pitch: Option<f32>,
    pub volume: Option<f32>,
    pub intensity: f32,
    pub random_draws: u8,
}

impl ConfusionState {
    pub fn tick(
        &mut self,
        marked_inside: bool,
        disallowed_screen_open: bool,
        container_open: bool,
        mut next_float: impl FnMut() -> f32,
    ) -> ConfusionTick {
        let trigger = marked_inside && self.intensity == 0.0;
        let pitch = trigger.then(|| 0.8 + 0.4 * next_float());
        self.intensity = if marked_inside {
            (self.intensity + 0.0125).clamp(0.0, 1.0)
        } else {
            (self.intensity - 0.05).clamp(0.0, 1.0)
        };
        ConfusionTick {
            close_disallowed_screen: marked_inside && disallowed_screen_open,
            close_open_container: marked_inside && disallowed_screen_open && container_open,
            play_trigger_sound: trigger,
            pitch,
            volume: trigger.then_some(0.25),
            intensity: self.intensity,
            random_draws: u8::from(trigger),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CrossKeyAdmission {
    pub destination_is_nether: bool,
    pub allow_entering_nether_using_portals: bool,
    pub same_key: bool,
    pub entity_can_teleport: bool,
    pub literal_end_to_overworld: bool,
    pub direct_unseen_credits_player: bool,
    pub is_ender_pearl: bool,
    pub pearl_owner_is_server_player: bool,
    pub pearl_owner_seen_credits: bool,
}

pub const fn admits_destination(input: CrossKeyAdmission) -> bool {
    if input.destination_is_nether && !input.allow_entering_nether_using_portals {
        return false;
    }
    if input.same_key {
        return true;
    }
    if !input.entity_can_teleport {
        return false;
    }
    if input.literal_end_to_overworld && input.direct_unseen_credits_player {
        return false;
    }
    if input.literal_end_to_overworld && input.is_ender_pearl {
        return input.pearl_owner_is_server_player && input.pearl_owner_seen_credits;
    }
    true
}
