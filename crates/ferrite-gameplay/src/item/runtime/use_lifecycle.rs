//! Shared component dispatch and active item-use lifecycle.

use crate::item::runtime::stack::ItemStack;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    Main,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsumableUse {
    pub duration_ticks: u32,
    pub can_always_eat: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UseComponents {
    pub consumable: Option<ConsumableUse>,
    pub swappable_equippable: bool,
    pub blocks_attacks: bool,
    pub kinetic_weapon: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseDispatch {
    Pass,
    InstantConsume,
    Start { duration_ticks: u32 },
}

pub fn dispatch(components: UseComponents, can_eat: bool) -> UseDispatch {
    if let Some(consumable) = components.consumable
        && (can_eat || consumable.can_always_eat)
    {
        return if consumable.duration_ticks == 0 {
            UseDispatch::InstantConsume
        } else {
            UseDispatch::Start {
                duration_ticks: consumable.duration_ticks,
            }
        };
    }
    if components.swappable_equippable {
        return UseDispatch::InstantConsume;
    }
    if components.blocks_attacks || components.kinetic_weapon {
        return UseDispatch::Start {
            duration_ticks: u32::MAX,
        };
    }
    UseDispatch::Pass
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UseProfile {
    pub duration_ticks: u32,
    pub release_driven: bool,
    pub consumable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveUse {
    pub hand: Hand,
    pub captured: ItemStack,
    pub captured_duration: u32,
    pub remaining: u32,
}

impl ActiveUse {
    pub fn start(
        current: &ItemStack,
        hand: Hand,
        profile: UseProfile,
        already_active: bool,
    ) -> Option<Self> {
        if current.is_empty() || already_active {
            return None;
        }
        Some(Self {
            hand,
            captured: current.clone(),
            captured_duration: profile.duration_ticks,
            remaining: profile.duration_ticks,
        })
    }

    pub const fn used_ticks(&self) -> u32 {
        self.captured_duration.saturating_sub(self.remaining)
    }

    pub fn tick(
        &mut self,
        current_before_tick: &ItemStack,
        current_after_tick: &ItemStack,
        profile: UseProfile,
        server_side: bool,
    ) -> TickOutcome {
        if !self.captured.same_item(current_before_tick) {
            return TickOutcome::StoppedDifferentItem;
        }

        self.captured = current_before_tick.clone();
        let observed_remaining = self.remaining;
        let periodic_consume = profile.consumable
            && self.used_ticks() > ((profile.duration_ticks as f64 * 0.218_75_f64).floor() as u32)
            && self.remaining.is_multiple_of(4);
        self.remaining = self.remaining.saturating_sub(1);

        let terminal = if self.remaining == 0 && server_side && !profile.release_driven {
            if self.captured.equal_stack(current_after_tick) {
                Some(TerminalUse::Finish)
            } else {
                Some(TerminalUse::Release)
            }
        } else {
            None
        };
        TickOutcome::Updated {
            observed_remaining,
            periodic_consume,
            terminal,
        }
    }

    pub fn release(
        &mut self,
        current: &ItemStack,
        release_succeeded: bool,
        profile: UseProfile,
    ) -> ReleaseOutcome {
        if !self.captured.same_item(current) {
            return ReleaseOutcome {
                invoked_release: false,
                apply_after_use: false,
                final_update_remaining: None,
            };
        }
        self.captured = current.clone();
        ReleaseOutcome {
            invoked_release: true,
            apply_after_use: release_succeeded,
            final_update_remaining: profile.release_driven.then_some(self.remaining),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalUse {
    Finish,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickOutcome {
    StoppedDifferentItem,
    Updated {
        observed_remaining: u32,
        periodic_consume: bool,
        terminal: Option<TerminalUse>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleaseOutcome {
    pub invoked_release: bool,
    pub apply_after_use: bool,
    pub final_update_remaining: Option<u32>,
}

pub fn seconds_to_ticks(seconds: f32) -> i32 {
    (seconds * 20.0_f32) as i32
}
