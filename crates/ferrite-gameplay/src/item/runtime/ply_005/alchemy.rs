//! Dragon-breath bottle acquisition and Water-potion block transactions.

pub const DRAGON_BREATH_ITEM_ID: u32 = 1_320;
pub const DRAGON_CLOUD_QUERY_INFLATION: f32 = 2.0;
pub const DRAGON_CLOUD_RADIUS_COST: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragonCloud {
    pub alive: bool,
    pub owner_is_ender_dragon: bool,
    pub radius: f32,
}

pub fn first_dragon_cloud(clouds: &[DragonCloud]) -> Option<usize> {
    clouds
        .iter()
        .position(|cloud| cloud.alive && cloud.owner_is_ender_dragon)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragonBreathFill {
    pub new_radius: f32,
    pub consumed_bottle: u8,
    pub item_used_stat: bool,
    pub interaction_criterion: bool,
    pub fluid_pickup_event: bool,
    pub add_default_breath: bool,
}

pub fn fill_dragon_breath(
    radius: f32,
    infinite_materials: bool,
    equal_breath_in_inventory: bool,
) -> DragonBreathFill {
    DragonBreathFill {
        new_radius: (radius - DRAGON_CLOUD_RADIUS_COST).clamp(0.0, 32.0),
        consumed_bottle: u8::from(!infinite_materials),
        item_used_stat: true,
        interaction_criterion: true,
        fluid_pickup_event: true,
        add_default_breath: !infinite_materials || !equal_breath_in_inventory,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrewingBottle {
    SplashWithHolder(u32),
    SplashWithoutHolder,
    PotionWithHolder(u32),
    LingeringWithHolder(u32),
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragonBreathBrew {
    pub bottles: [BrewingBottle; 3],
    pub ingredient_consumed: u8,
    pub brew_event: bool,
}

pub fn brew_dragon_breath(mut bottles: [BrewingBottle; 3]) -> DragonBreathBrew {
    for bottle in &mut bottles {
        if let BrewingBottle::SplashWithHolder(holder) = *bottle {
            *bottle = BrewingBottle::LingeringWithHolder(holder);
        }
    }
    DragonBreathBrew {
        bottles,
        ingredient_consumed: 1,
        brew_event: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CauldronResult {
    Success,
    TryWithEmptyHand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaterCauldronPour {
    pub result: CauldronResult,
    pub next_level: u8,
    pub transforms_to_bottle: bool,
    pub item_used_stat: bool,
    pub use_cauldron_stat: bool,
    pub fluid_place_event: bool,
}

pub const fn pour_water_cauldron(
    current_level: u8,
    water_predicate: bool,
    server_side: bool,
) -> WaterCauldronPour {
    if current_level >= 3 || !water_predicate {
        return WaterCauldronPour {
            result: CauldronResult::TryWithEmptyHand,
            next_level: current_level,
            transforms_to_bottle: false,
            item_used_stat: false,
            use_cauldron_stat: false,
            fluid_place_event: false,
        };
    }
    WaterCauldronPour {
        result: CauldronResult::Success,
        next_level: current_level + 1,
        transforms_to_bottle: server_side,
        item_used_stat: server_side,
        use_cauldron_stat: server_side,
        fluid_place_event: server_side,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MudConversion {
    pub success: bool,
    pub transforms_to_bottle: bool,
    pub splash_random_doubles: u8,
    pub writes_mud: bool,
    pub write_result_observed: bool,
}

pub const fn convert_to_mud(
    face_is_down: bool,
    convertible_tag_member: bool,
    water_predicate: bool,
    server_side: bool,
) -> MudConversion {
    let success = !face_is_down && convertible_tag_member && water_predicate;
    MudConversion {
        success,
        transforms_to_bottle: success,
        splash_random_doubles: if success && server_side { 10 } else { 0 },
        writes_mud: success && server_side,
        write_result_observed: false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreativePourRemainder {
    pub retains_potion: bool,
    pub attempts_bottle_insert: bool,
}

pub const fn creative_pour_remainder(equal_bottle_in_inventory: bool) -> CreativePourRemainder {
    CreativePourRemainder {
        retains_potion: true,
        attempts_bottle_insert: !equal_bottle_in_inventory,
    }
}
