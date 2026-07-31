//! Seeded client sound resolution, gain, attenuation, and delay semantics.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoundRequest {
    pub original_volume: f32,
    pub pitch: f32,
    pub final_category_volume: f32,
    pub category_gain: f32,
    pub resource_attenuation_distance: f32,
    pub seed: i64,
    pub music: bool,
    pub permits_silent_start: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoundAvailability {
    pub resources_loaded: bool,
    pub allowed: bool,
    pub known_event: bool,
    pub intentionally_empty: bool,
    pub event_has_variants: bool,
    pub channel_available: bool,
}

impl Default for SoundAvailability {
    fn default() -> Self {
        Self {
            resources_loaded: true,
            allowed: true,
            known_event: true,
            intentionally_empty: false,
            event_has_variants: true,
            channel_available: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StartedSound {
    pub seed: i64,
    pub pitch: f32,
    pub gain: f32,
    pub attenuation_distance: f32,
    pub retained_until_tick: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundRejection {
    ResourcesUnloaded,
    Disallowed,
    UnknownEvent,
    IntentionallyEmpty,
    EmptyEvent,
    Silent,
    ChannelUnavailable,
}

pub fn start_sound(
    request: SoundRequest,
    availability: SoundAvailability,
    sound_tick: u64,
) -> Result<StartedSound, SoundRejection> {
    if !availability.resources_loaded {
        return Err(SoundRejection::ResourcesUnloaded);
    }
    if !availability.allowed {
        return Err(SoundRejection::Disallowed);
    }
    if !availability.known_event {
        return Err(SoundRejection::UnknownEvent);
    }
    if availability.intentionally_empty {
        return Err(SoundRejection::IntentionallyEmpty);
    }
    if !availability.event_has_variants {
        return Err(SoundRejection::EmptyEvent);
    }
    let gain = request.original_volume.clamp(0.0, 1.0)
        * request.final_category_volume.clamp(0.0, 1.0)
        * request.category_gain;
    if gain == 0.0 && !request.music && !request.permits_silent_start {
        return Err(SoundRejection::Silent);
    }
    if !availability.channel_available {
        return Err(SoundRejection::ChannelUnavailable);
    }
    Ok(StartedSound {
        seed: request.seed,
        pitch: request.pitch.clamp(0.5, 2.0),
        gain,
        attenuation_distance: request.original_volume.max(1.0)
            * request.resource_attenuation_distance,
        retained_until_tick: sound_tick.saturating_add(20),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalSoundSchedule {
    pub seed: i64,
    pub consumed_client_next_long: bool,
    pub delay_ticks: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityBoundSound {
    pub entity_id: i32,
    pub seed: i64,
}

#[must_use]
pub const fn entity_bound_sound(
    entity_id: i32,
    entity_present: bool,
    seed: i64,
) -> Option<EntityBoundSound> {
    if entity_present {
        Some(EntityBoundSound { entity_id, seed })
    } else {
        None
    }
}

#[must_use]
pub fn schedule_local_sound(
    supplied_seed: Option<i64>,
    next_client_long: i64,
    distance_squared: f64,
    distance_delay: bool,
) -> LocalSoundSchedule {
    let distance = distance_squared.max(0.0).sqrt();
    let delay_ticks = if distance_delay && distance_squared > 100.0 {
        ((distance / 40.0) * 20.0).floor() as u32
    } else {
        0
    };
    LocalSoundSchedule {
        seed: supplied_seed.unwrap_or(next_client_long),
        consumed_client_next_long: supplied_seed.is_none(),
        delay_ticks,
    }
}
