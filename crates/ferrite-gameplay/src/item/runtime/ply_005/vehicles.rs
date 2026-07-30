//! Minecart placement, subtype interaction, activation, and itemization.

pub const FURNACE_FUEL_INCREMENT: u32 = 3_600;
pub const FURNACE_FUEL_CAP: u32 = 32_000;
pub const TNT_ACTIVATOR_FUSE: u32 = 80;
pub const COMMAND_ACTIVATION_THROTTLE: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinecartKind {
    Ordinary,
    Chest,
    Furnace,
    Tnt,
    Hopper,
    CommandBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailShape {
    Flat,
    Ascending,
    TagInjectedNonRail,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinecartPlacement {
    pub success: bool,
    pub vertical_offset: f64,
    pub spawn_reason_dispenser: bool,
    pub configuration_before_explicit_entity_data: bool,
    pub consumed: u8,
    pub placement_event: bool,
    pub admission_result_observed: bool,
}

pub const fn place_minecart(
    rail_tag_member: bool,
    shape: RailShape,
    factory_created: bool,
    improvements_enabled: bool,
    intersecting_minecart: bool,
    server_side: bool,
) -> MinecartPlacement {
    let collision_rejected = improvements_enabled && intersecting_minecart;
    if !rail_tag_member || !factory_created || collision_rejected {
        return MinecartPlacement {
            success: false,
            vertical_offset: 0.0,
            spawn_reason_dispenser: true,
            configuration_before_explicit_entity_data: false,
            consumed: 0,
            placement_event: false,
            admission_result_observed: false,
        };
    }
    MinecartPlacement {
        success: true,
        vertical_offset: if matches!(shape, RailShape::Ascending) {
            0.5625
        } else {
            0.0625
        },
        spawn_reason_dispenser: true,
        configuration_before_explicit_entity_data: true,
        consumed: 1,
        placement_event: server_side,
        admission_result_observed: false,
    }
}

pub const fn dispenser_vertical_offset(
    front_is_rail: bool,
    front_is_air: bool,
    lower_is_rail: bool,
    rail_ascending: bool,
    facing_down: bool,
) -> Option<f64> {
    if front_is_rail {
        Some(if rail_ascending { 0.6 } else { 0.1 })
    } else if front_is_air && lower_is_rail {
        Some(if !facing_down && rail_ascending {
            -0.4
        } else {
            -0.9
        })
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrdinaryCartInteraction {
    pub passenger_installed: bool,
    pub literal_success: bool,
    pub start_riding_calls: u8,
}

pub const fn interact_ordinary_cart(
    client_side: bool,
    empty: bool,
    secondary_use: bool,
    first_start_riding: bool,
) -> OrdinaryCartInteraction {
    if !empty || secondary_use {
        return OrdinaryCartInteraction {
            passenger_installed: false,
            literal_success: false,
            start_riding_calls: 0,
        };
    }
    if client_side {
        return OrdinaryCartInteraction {
            passenger_installed: false,
            literal_success: true,
            start_riding_calls: 0,
        };
    }
    OrdinaryCartInteraction {
        passenger_installed: first_start_riding,
        literal_success: false,
        start_riding_calls: if first_start_riding { 2 } else { 1 },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FurnaceFuelOutcome {
    pub action_consumed: bool,
    pub stack_consumed: u8,
    pub new_fuel: u32,
    pub push_updated: bool,
}

pub fn fuel_furnace_cart(current_fuel: u32, live_fuel_tag_member: bool) -> FurnaceFuelOutcome {
    let admitted = live_fuel_tag_member
        && current_fuel
            .checked_add(FURNACE_FUEL_INCREMENT)
            .is_some_and(|fuel| fuel <= FURNACE_FUEL_CAP);
    FurnaceFuelOutcome {
        action_consumed: true,
        stack_consumed: u8::from(admitted),
        new_fuel: if admitted {
            current_fuel + FURNACE_FUEL_INCREMENT
        } else {
            current_fuel
        },
        push_updated: admitted,
    }
}

pub const fn hopper_enabled(powered_activator: bool) -> bool {
    !powered_activator
}

pub const fn tnt_fuse(current_fuse: i32, powered_activator: bool) -> i32 {
    if powered_activator && current_fuse < 0 {
        TNT_ACTIVATOR_FUSE as i32
    } else {
        current_fuse
    }
}

pub const fn command_cart_may_activate(
    current_tick: u32,
    last_activation_tick: u32,
    powered_activator: bool,
) -> bool {
    powered_activator
        && current_tick.saturating_sub(last_activation_tick) >= COMMAND_ACTIVATION_THROTTLE
}

pub const fn destruction_item(kind: MinecartKind) -> MinecartKind {
    if matches!(kind, MinecartKind::CommandBlock) {
        MinecartKind::Ordinary
    } else {
        kind
    }
}

pub const fn pick_item(kind: MinecartKind) -> MinecartKind {
    kind
}

pub const fn scatters_contents(kind: MinecartKind, destructive_removal: bool) -> bool {
    destructive_removal && matches!(kind, MinecartKind::Chest | MinecartKind::Hopper)
}
