//! Agricultural ground, dirt substrates, Nylium, Moss, Mud, and ice boundaries.

use ferrite_foundation::direction::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgriculturalGround {
    DirtPath,
    Farmland { moisture: u8 },
}

impl AgriculturalGround {
    pub fn state_id(self) -> Option<u32> {
        match self {
            Self::DirtPath => Some(14_815),
            Self::Farmland { moisture } if moisture <= 7 => Some(5_319 + moisture as u32),
            Self::Farmland { .. } => None,
        }
    }

    pub fn schedule_support_loss(self, direction: Direction, survives: bool) -> bool {
        matches!(direction, Direction::Up) && !survives
    }

    pub fn due_converts_to_dirt(self, survives_now: bool) -> bool {
        match self {
            Self::DirtPath => true,
            Self::Farmland { .. } => !survives_now,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoistureTick {
    NoChange,
    Moisture(u8),
    Dirt,
}

pub fn farmland_random_tick(
    moisture: u8,
    water_or_rain: bool,
    above_maintains_farmland: bool,
) -> MoistureTick {
    if water_or_rain {
        if moisture < 7 {
            MoistureTick::Moisture(7)
        } else {
            MoistureTick::NoChange
        }
    } else if moisture > 0 {
        MoistureTick::Moisture(moisture - 1)
    } else if above_maintains_farmland {
        MoistureTick::NoChange
    } else {
        MoistureTick::Dirt
    }
}

pub fn farmland_tramples(
    next_float: f32,
    fall_distance: f32,
    living: bool,
    player: bool,
    mob_griefing: bool,
    width: f32,
    height: f32,
) -> bool {
    next_float < fall_distance - 0.5
        && living
        && (player || mob_griefing)
        && width * width * height > 0.512
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirtKind {
    Grass,
    Dirt,
    CoarseDirt,
    Podzol,
    Mycelium,
    RootedDirt,
}

impl DirtKind {
    pub fn state_id(self, snowy: bool) -> u32 {
        match self {
            Self::Grass => 8 + u32::from(!snowy),
            Self::Dirt => 10,
            Self::CoarseDirt => 11,
            Self::Podzol => 12 + u32::from(!snowy),
            Self::Mycelium => 8_918 + u32::from(!snowy),
            Self::RootedDirt => 30_414,
        }
    }

    pub fn has_snowy_state(self) -> bool {
        matches!(self, Self::Grass | Self::Podzol | Self::Mycelium)
    }
}

pub fn snowy_after_update(
    kind: DirtKind,
    direction: Direction,
    previous: bool,
    above_in_snow_tag: bool,
) -> bool {
    if kind.has_snowy_state() && matches!(direction, Direction::Up) {
        above_in_snow_tag
    } else {
        previous
    }
}

pub fn spreading_ground_survives(
    above_exact_one_layer_snow: bool,
    above_full_fluid: bool,
    light_dampening_into: u8,
) -> bool {
    above_exact_one_layer_snow || (!above_full_fluid && light_dampening_into < 15)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpreadTick {
    MissingDirtRegistry,
    DecayToDirt,
    TooDark,
    FourAttempts,
}

pub fn spreading_ground_tick(
    dirt_registry_present: bool,
    survives: bool,
    maximum_raw_brightness_above: u8,
) -> SpreadTick {
    if !dirt_registry_present {
        SpreadTick::MissingDirtRegistry
    } else if !survives {
        SpreadTick::DecayToDirt
    } else if maximum_raw_brightness_above < 9 {
        SpreadTick::TooDark
    } else {
        SpreadTick::FourAttempts
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolTransform {
    DirtPath,
    Farmland,
    Dirt,
    Mud,
}

pub fn shovel_transform(
    kind: DirtKind,
    clicked_face: Direction,
    above_air: bool,
) -> Option<ToolTransform> {
    if matches!(clicked_face, Direction::Down) || !above_air {
        None
    } else {
        let _ = kind;
        Some(ToolTransform::DirtPath)
    }
}

pub fn hoe_transform(
    kind: DirtKind,
    clicked_face: Direction,
    above_air: bool,
) -> Option<ToolTransform> {
    match kind {
        DirtKind::Grass | DirtKind::Dirt
            if !matches!(clicked_face, Direction::Down) && above_air =>
        {
            Some(ToolTransform::Farmland)
        }
        DirtKind::CoarseDirt if !matches!(clicked_face, Direction::Down) && above_air => {
            Some(ToolTransform::Dirt)
        }
        DirtKind::RootedDirt => Some(ToolTransform::Dirt),
        _ => None,
    }
}

pub fn water_bottle_to_mud(kind: DirtKind, clicked_face: Direction) -> bool {
    !matches!(clicked_face, Direction::Down)
        && matches!(
            kind,
            DirtKind::Dirt | DirtKind::CoarseDirt | DirtKind::RootedDirt
        )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nylium {
    Crimson,
    Warped,
}

impl Nylium {
    pub fn state_id(self) -> u32 {
        match self {
            Self::Crimson => 20_974,
            Self::Warped => 20_957,
        }
    }

    pub fn random_tick_decays(self, light_dampening_into: u8) -> bool {
        let _ = self;
        light_dampening_into == 15
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NyliumBoneMeal {
    pub primary_feature: bool,
    pub sprouts_feature: bool,
    pub twisting_feature: bool,
}

pub fn nylium_bone_meal(kind: Nylium, next_int_8: Option<u8>) -> NyliumBoneMeal {
    match kind {
        Nylium::Crimson => NyliumBoneMeal {
            primary_feature: true,
            sprouts_feature: false,
            twisting_feature: false,
        },
        Nylium::Warped => NyliumBoneMeal {
            primary_feature: true,
            sprouts_feature: true,
            twisting_feature: next_int_8.is_some_and(|draw| draw == 0),
        },
    }
}

pub fn netherrack_conversion(
    saw_crimson: bool,
    saw_warped: bool,
    next_boolean: Option<bool>,
) -> Option<Nylium> {
    match (saw_crimson, saw_warped) {
        (true, true) => Some(if next_boolean.unwrap_or(false) {
            Nylium::Warped
        } else {
            Nylium::Crimson
        }),
        (true, false) => Some(Nylium::Crimson),
        (false, true) => Some(Nylium::Warped),
        (false, false) => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MossKind {
    Block,
    Carpet,
}

impl MossKind {
    pub fn state_id(self) -> u32 {
        match self {
            Self::Block => 30_355,
            Self::Carpet => 30_306,
        }
    }

    pub fn survives(self, below_is_air: bool) -> bool {
        match self {
            Self::Block => true,
            Self::Carpet => !below_is_air,
        }
    }

    pub fn compost_chance(self) -> f32 {
        match self {
            Self::Block => 0.65,
            Self::Carpet => 0.3,
        }
    }
}

pub fn mud_dripstone_converts_to_clay(transfer_draw: f32) -> bool {
    transfer_draw < 0.17578125
}

pub fn packed_ice_friction() -> f32 {
    0.98
}
