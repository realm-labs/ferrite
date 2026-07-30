//! Common vehicle hurt, destruction, itemization, and decay.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VehicleDamageInput {
    pub removed: bool,
    pub invulnerable: bool,
    pub mob_explosion: bool,
    pub mob_griefing: bool,
    pub amount: f32,
    pub hurt_direction: i8,
    pub accumulated_damage: f32,
    pub creative_attacker: bool,
    pub source_forces_destruction: bool,
    pub entity_drops: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VehicleDamageOutcome {
    pub admitted: bool,
    pub hurt_direction: i8,
    pub hurt_time: u8,
    pub accumulated_damage: f32,
    pub marked_hurt: bool,
    pub damage_event: bool,
    pub destroyed: bool,
    pub discarded: bool,
    pub itemized: bool,
    pub copies_custom_name: bool,
}

#[must_use]
pub fn damage_vehicle(input: VehicleDamageInput) -> VehicleDamageOutcome {
    if input.removed || input.invulnerable || (input.mob_explosion && !input.mob_griefing) {
        return VehicleDamageOutcome {
            admitted: false,
            hurt_direction: input.hurt_direction,
            hurt_time: 0,
            accumulated_damage: input.accumulated_damage,
            marked_hurt: false,
            damage_event: false,
            destroyed: false,
            discarded: false,
            itemized: false,
            copies_custom_name: false,
        };
    }

    let accumulated_damage = input.accumulated_damage + input.amount * 10.0;
    let threshold_reached = accumulated_damage > 40.0 || input.source_forces_destruction;
    let discarded = input.creative_attacker && !input.source_forces_destruction;
    let destroyed = threshold_reached && !discarded;
    let itemized = destroyed && input.entity_drops;
    VehicleDamageOutcome {
        admitted: true,
        hurt_direction: -input.hurt_direction,
        hurt_time: 10,
        accumulated_damage,
        marked_hurt: true,
        damage_event: true,
        destroyed,
        discarded,
        itemized,
        copies_custom_name: itemized,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VehicleDamageDecay {
    pub hurt_time: u8,
    pub accumulated_damage: f32,
}

#[must_use]
pub fn decay_vehicle_damage(hurt_time: u8, accumulated_damage: f32) -> VehicleDamageDecay {
    VehicleDamageDecay {
        hurt_time: hurt_time.saturating_sub(1),
        accumulated_damage: if accumulated_damage > 0.0 {
            accumulated_damage - 1.0
        } else {
            accumulated_damage
        },
    }
}
