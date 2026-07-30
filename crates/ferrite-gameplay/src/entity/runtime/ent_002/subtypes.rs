//! Minecart subtype fuel, fuse, activation, inventory, and ticker hooks.

use crate::entity::runtime::ent_002::boat::Vector3;

pub const FURNACE_FUEL_INCREMENT: u32 = 3_600;
pub const FURNACE_FUEL_CAP: u32 = 32_000;
pub const TNT_FUSE_TICKS: i32 = 80;
pub const COMMAND_ACTIVATION_INTERVAL: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RideableActivation {
    pub eject_passengers: bool,
    pub damage: u8,
}

#[must_use]
pub const fn activate_rideable(powered: bool) -> RideableActivation {
    RideableActivation {
        eject_passengers: powered,
        damage: if powered { 50 } else { 0 },
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FurnaceFuel {
    pub fuel: u32,
    pub consumed: u8,
    pub push_x: f64,
    pub push_z: f64,
}

#[must_use]
pub fn fuel_furnace(
    current_fuel: u32,
    live_fuel_tag_member: bool,
    cart_x: f64,
    cart_z: f64,
    player_x: f64,
    player_z: f64,
) -> FurnaceFuel {
    let admitted = live_fuel_tag_member
        && current_fuel
            .checked_add(FURNACE_FUEL_INCREMENT)
            .is_some_and(|fuel| fuel <= FURNACE_FUEL_CAP);
    FurnaceFuel {
        fuel: if admitted {
            current_fuel + FURNACE_FUEL_INCREMENT
        } else {
            current_fuel
        },
        consumed: u8::from(admitted),
        push_x: if admitted { cart_x - player_x } else { 0.0 },
        push_z: if admitted { cart_z - player_z } else { 0.0 },
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FurnaceStep {
    pub fuel: u32,
    pub velocity: Vector3,
    pub lit: bool,
}

#[must_use]
pub fn furnace_step(fuel: u32, push_x: f64, push_z: f64, velocity: Vector3) -> FurnaceStep {
    let fuel = fuel.saturating_sub(1);
    let push_length = push_x.hypot(push_z);
    let velocity = if push_length > 0.0001 {
        Vector3::new(
            velocity.x * 0.8 + push_x / push_length,
            0.0,
            velocity.z * 0.8 + push_z / push_length,
        )
    } else {
        Vector3::new(velocity.x * 0.98, 0.0, velocity.z * 0.98)
    };
    FurnaceStep {
        fuel,
        velocity,
        lit: fuel > 0,
    }
}

#[must_use]
pub const fn furnace_maximum_speed(family_maximum: f64, in_water: bool) -> f64 {
    let speed = family_maximum * 0.5;
    if in_water { speed * 0.5 } else { speed }
}

#[must_use]
pub const fn prime_tnt(fuse: i32, powered_activator: bool) -> i32 {
    if powered_activator && fuse < 0 {
        TNT_FUSE_TICKS
    } else {
        fuse
    }
}

#[must_use]
pub const fn tnt_collision_explodes(fuse: i32, horizontal_speed_squared: f64) -> bool {
    fuse >= 0 && horizontal_speed_squared >= 0.01
}

#[must_use]
pub const fn shortened_tnt_fuse(first_draw_twenty: u8, second_draw_twenty: u8) -> i32 {
    (first_draw_twenty % 20 + second_draw_twenty % 20) as i32
}

#[must_use]
pub fn tnt_explosion_power(
    tnt_explodes: bool,
    base: f64,
    factor: f64,
    draw: f64,
    horizontal_speed_squared: f64,
) -> Option<f64> {
    tnt_explodes.then(|| base + factor * draw * 1.5 * horizontal_speed_squared.sqrt().min(5.0))
}

#[must_use]
pub const fn hopper_enabled(powered_activator: bool) -> bool {
    !powered_activator
}

#[must_use]
pub const fn command_cart_activates(
    powered_activator: bool,
    current_tick: u32,
    last_activation_tick: u32,
) -> bool {
    powered_activator
        && current_tick.saturating_sub(last_activation_tick) >= COMMAND_ACTIVATION_INTERVAL
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubtypeHook {
    HopperPickup,
    SpawnerTick,
    CommandControl,
    ContainerMenu,
    ContainerLoot,
    ContainerDrop,
}

#[must_use]
pub const fn subtype_hooks(
    hopper: bool,
    spawner: bool,
    command: bool,
    container: bool,
    destructive_removal: bool,
) -> [Option<SubtypeHook>; 6] {
    [
        if hopper {
            Some(SubtypeHook::HopperPickup)
        } else {
            None
        },
        if spawner {
            Some(SubtypeHook::SpawnerTick)
        } else {
            None
        },
        if command {
            Some(SubtypeHook::CommandControl)
        } else {
            None
        },
        if container {
            Some(SubtypeHook::ContainerMenu)
        } else {
            None
        },
        if container {
            Some(SubtypeHook::ContainerLoot)
        } else {
            None
        },
        if container && destructive_removal {
            Some(SubtypeHook::ContainerDrop)
        } else {
            None
        },
    ]
}
