//! Server effect cadence, duration, hidden promotion, refresh, and attribute plans.

use crate::entity::runtime::ent_006::instance::{EffectInstance, INFINITE_DURATION};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickRemoval {
    NoDuration,
    CallbackRejected,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectTick {
    pub apply_callback: bool,
    pub remove: Option<TickRemoval>,
    pub promoted_hidden: bool,
    pub update_with_attribute_refresh: bool,
    pub periodic_update_without_refresh: bool,
}

#[must_use]
pub fn tick_instance(
    instance: &mut EffectInstance,
    entity_tick_count: u32,
    cadence_scheduled: bool,
    callback_result: bool,
) -> EffectTick {
    if !instance.has_duration() {
        return EffectTick {
            apply_callback: false,
            remove: Some(TickRemoval::NoDuration),
            promoted_hidden: false,
            update_with_attribute_refresh: false,
            periodic_update_without_refresh: false,
        };
    }
    let cadence_value = if instance.duration == INFINITE_DURATION {
        entity_tick_count as i32
    } else {
        instance.duration
    };
    let apply_callback = cadence_scheduled && cadence_value >= 0;
    if apply_callback && !callback_result {
        return EffectTick {
            apply_callback: true,
            remove: Some(TickRemoval::CallbackRejected),
            promoted_hidden: false,
            update_with_attribute_refresh: false,
            periodic_update_without_refresh: false,
        };
    }

    decrement_hidden(instance.hidden.as_mut());
    if instance.duration > 0 {
        instance.duration -= 1;
    }
    let mut promoted_hidden = false;
    if instance.duration == 0
        && let Some(hidden) = instance.hidden.take()
    {
        *instance = *hidden;
        promoted_hidden = true;
    }
    let remains = instance.has_duration();
    EffectTick {
        apply_callback,
        remove: (!remains).then_some(TickRemoval::Expired),
        promoted_hidden,
        update_with_attribute_refresh: promoted_hidden,
        periodic_update_without_refresh: remains
            && !promoted_hidden
            && instance.duration > 0
            && instance.duration % 600 == 0,
    }
}

fn decrement_hidden(hidden: Option<&mut Box<EffectInstance>>) {
    let Some(hidden) = hidden else {
        return;
    };
    decrement_hidden(hidden.hidden.as_mut());
    if hidden.duration > 0 {
        hidden.duration -= 1;
    }
}

#[must_use]
pub const fn cadence_value(instance: &EffectInstance, entity_tick_count: u32) -> i32 {
    if instance.duration == INFINITE_DURATION {
        entity_tick_count as i32
    } else {
        instance.duration
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttributeRefresh {
    pub remove_modifier_by_id_first: bool,
    pub permanent_amount: f64,
    pub clamp_health_and_absorption: bool,
    pub refresh_dimensions: bool,
    pub refresh_waypoint_tracking: bool,
}

#[must_use]
pub const fn attribute_refresh(base_amount: f64, amplifier: u8) -> AttributeRefresh {
    AttributeRefresh {
        remove_modifier_by_id_first: true,
        permanent_amount: base_amount * (amplifier as f64 + 1.0),
        clamp_health_and_absorption: true,
        refresh_dimensions: true,
        refresh_waypoint_tracking: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickPass {
    pub visited: usize,
    pub aborted_on_concurrent_modification: bool,
    pub deferred: usize,
}

#[must_use]
pub const fn tick_pass(active_count: usize, mutation_at_index: Option<usize>) -> TickPass {
    match mutation_at_index {
        Some(index) if index < active_count => TickPass {
            visited: index + 1,
            aborted_on_concurrent_modification: true,
            deferred: active_count - index - 1,
        },
        _ => TickPass {
            visited: active_count,
            aborted_on_concurrent_modification: false,
            deferred: 0,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrdinaryRemoval {
    pub remove_from_map: bool,
    pub removal_callbacks: u8,
    pub remove_attribute_modifiers: bool,
}

#[must_use]
pub const fn ordinary_removal(present: bool) -> OrdinaryRemoval {
    OrdinaryRemoval {
        remove_from_map: present,
        removal_callbacks: if present { 1 } else { 0 },
        remove_attribute_modifiers: present,
    }
}
