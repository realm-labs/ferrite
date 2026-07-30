//! Effect construction, hidden-chain merge, ordinary add, force-add, and removal plans.

pub const INFINITE_DURATION: i32 = -1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectInstance {
    pub duration: i32,
    pub amplifier: u8,
    pub ambient: bool,
    pub visible_particles: bool,
    pub show_icon: bool,
    pub hidden: Option<Box<Self>>,
    pub blend_state: u32,
}

impl EffectInstance {
    #[must_use]
    pub fn new(
        duration: i32,
        amplifier: u16,
        ambient: bool,
        visible_particles: bool,
        show_icon: bool,
    ) -> Self {
        Self {
            duration,
            amplifier: amplifier.min(255) as u8,
            ambient,
            visible_particles,
            show_icon,
            hidden: None,
            blend_state: 0,
        }
    }

    #[must_use]
    pub const fn has_duration(&self) -> bool {
        self.duration == INFINITE_DURATION || self.duration > 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MergeOutcome {
    pub changed: bool,
    pub amplifier_or_duration_changed: bool,
    pub flags_changed: bool,
}

#[must_use]
pub fn merge(current: &mut EffectInstance, incoming: &EffectInstance) -> MergeOutcome {
    let mut changed = false;
    let mut strength_or_duration = false;
    if incoming.amplifier > current.amplifier {
        if duration_shorter(incoming.duration, current.duration) {
            let mut hidden = current.clone();
            hidden.hidden = current.hidden.take();
            current.hidden = Some(Box::new(hidden));
        }
        current.amplifier = incoming.amplifier;
        current.duration = incoming.duration;
        changed = true;
        strength_or_duration = true;
    } else if duration_longer(incoming.duration, current.duration) {
        if incoming.amplifier == current.amplifier {
            current.duration = incoming.duration;
            changed = true;
            strength_or_duration = true;
        } else if let Some(hidden) = current.hidden.as_mut() {
            let hidden_outcome = merge(hidden, incoming);
            changed |= hidden_outcome.changed;
        } else {
            current.hidden = Some(Box::new(incoming.clone()));
            changed = true;
        }
    }

    if !incoming.ambient && current.ambient {
        current.ambient = false;
        changed = true;
    } else if strength_or_duration && current.ambient != incoming.ambient {
        current.ambient = incoming.ambient;
        changed = true;
    }
    let mut flags_changed = false;
    if current.visible_particles != incoming.visible_particles {
        current.visible_particles = incoming.visible_particles;
        flags_changed = true;
    }
    if current.show_icon != incoming.show_icon {
        current.show_icon = incoming.show_icon;
        flags_changed = true;
    }
    changed |= flags_changed;
    MergeOutcome {
        changed,
        amplifier_or_duration_changed: strength_or_duration,
        flags_changed,
    }
}

fn duration_longer(candidate: i32, current: i32) -> bool {
    candidate == INFINITE_DURATION || (current != INFINITE_DURATION && candidate > current)
}

fn duration_shorter(candidate: i32, current: i32) -> bool {
    current == INFINITE_DURATION || (candidate != INFINITE_DURATION && candidate < current)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddEffectPlan {
    pub accepted: bool,
    pub map_changed: bool,
    pub living_add_callback: bool,
    pub living_update_with_attribute_refresh: bool,
    pub effect_added_callback: bool,
    pub effect_started_callback: bool,
    pub notify_player_passengers: bool,
}

#[must_use]
pub fn add_effect(
    current: Option<&mut EffectInstance>,
    incoming: &EffectInstance,
    applicable: bool,
) -> AddEffectPlan {
    if !applicable {
        return AddEffectPlan {
            accepted: false,
            map_changed: false,
            living_add_callback: false,
            living_update_with_attribute_refresh: false,
            effect_added_callback: false,
            effect_started_callback: false,
            notify_player_passengers: false,
        };
    }
    match current {
        None => AddEffectPlan {
            accepted: true,
            map_changed: true,
            living_add_callback: true,
            living_update_with_attribute_refresh: false,
            effect_added_callback: true,
            effect_started_callback: true,
            notify_player_passengers: true,
        },
        Some(current) => {
            let outcome = merge(current, incoming);
            AddEffectPlan {
                accepted: true,
                map_changed: outcome.changed,
                living_add_callback: false,
                living_update_with_attribute_refresh: outcome.changed,
                effect_added_callback: false,
                effect_started_callback: true,
                notify_player_passengers: false,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForceAddPlan {
    pub replace: bool,
    pub copy_blend_state: bool,
    pub living_add_callback: bool,
    pub living_update_callback: bool,
    pub effect_added_callback: bool,
    pub effect_started_callback: bool,
}

#[must_use]
pub const fn force_add_plan(replacing: bool) -> ForceAddPlan {
    ForceAddPlan {
        replace: replacing,
        copy_blend_state: replacing,
        living_add_callback: !replacing,
        living_update_callback: replacing,
        effect_added_callback: false,
        effect_started_callback: false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoveAllPlan {
    pub admitted: bool,
    pub copy_then_clear_map: bool,
    pub remove_modifiers_after_clear: bool,
}

#[must_use]
pub const fn remove_all_plan(server_side: bool, active_count: usize) -> RemoveAllPlan {
    RemoveAllPlan {
        admitted: server_side && active_count > 0,
        copy_then_clear_map: server_side && active_count > 0,
        remove_modifiers_after_clear: server_side && active_count > 0,
    }
}
