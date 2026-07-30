//! Mob-bucket capture, release, state transfer, and dispenser transactions.

pub const MOB_BUCKET_MAXIMUM_STACK: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobBucketKind {
    Pufferfish,
    Salmon,
    Cod,
    TropicalFish,
    Axolotl,
    SulfurCube,
    Tadpole,
}

impl MobBucketKind {
    pub const fn contains_water(self) -> bool {
        !matches!(self, Self::SulfurCube)
    }

    pub const fn entity_id(self) -> u32 {
        match self {
            Self::Pufferfish => 101,
            Self::Salmon => 115,
            Self::Cod => 27,
            Self::TropicalFish => 137,
            Self::Axolotl => 7,
            Self::SulfurCube => 130,
            Self::Tadpole => 131,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmptyBucketKind {
    Water,
    Dry,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureStep {
    PickupSound,
    WriteCapturedState,
    ReplaceHand,
    FilledBucketCriterion,
    DropLeash,
    DiscardMob,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureOutcome {
    pub success: bool,
    pub consumed_input: u8,
    pub retained_for_infinite_materials: bool,
    pub steps: Vec<CaptureStep>,
}

pub fn capture_mob(kind: MobBucketKind, alive: bool, held: EmptyBucketKind) -> CaptureOutcome {
    let expected = if kind.contains_water() {
        EmptyBucketKind::Water
    } else {
        EmptyBucketKind::Dry
    };
    if !alive || held != expected {
        return CaptureOutcome {
            success: false,
            consumed_input: 0,
            retained_for_infinite_materials: false,
            steps: Vec::new(),
        };
    }
    CaptureOutcome {
        success: true,
        consumed_input: 1,
        retained_for_infinite_materials: false,
        steps: vec![
            CaptureStep::PickupSound,
            CaptureStep::WriteCapturedState,
            CaptureStep::ReplaceHand,
            CaptureStep::FilledBucketCriterion,
            CaptureStep::DropLeash,
            CaptureStep::DiscardMob,
        ],
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CommonCapturedState {
    pub no_ai: bool,
    pub silent: bool,
    pub no_gravity: bool,
    pub glowing: bool,
    pub invulnerable: bool,
    pub persistence_required: bool,
    pub health: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariantTransfer {
    None,
    SalmonSize,
    TropicalPatternAndColors,
    AxolotlVariantAgeAndCooldown,
    TadpoleAge,
    SulfurContentAndAge,
}

pub const fn variant_transfer(kind: MobBucketKind) -> VariantTransfer {
    match kind {
        MobBucketKind::Pufferfish | MobBucketKind::Cod => VariantTransfer::None,
        MobBucketKind::Salmon => VariantTransfer::SalmonSize,
        MobBucketKind::TropicalFish => VariantTransfer::TropicalPatternAndColors,
        MobBucketKind::Axolotl => VariantTransfer::AxolotlVariantAgeAndCooldown,
        MobBucketKind::Tadpole => VariantTransfer::TadpoleAge,
        MobBucketKind::SulfurCube => VariantTransfer::SulfurContentAndAge,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementTarget {
    ClickedLiquidContainer,
    Adjacent,
}

pub const fn placement_target(
    contains_water: bool,
    clicked_accepts_liquid: bool,
) -> PlacementTarget {
    if contains_water && clicked_accepts_liquid {
        PlacementTarget::ClickedLiquidContainer
    } else {
        PlacementTarget::Adjacent
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleaseInput {
    pub permissions_admitted: bool,
    pub fluid_admitted: bool,
    pub evaporating_dimension: bool,
    pub server_side: bool,
    pub factory_created: bool,
    pub admission_accepted: bool,
    pub infinite_materials: bool,
    pub dispenser: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleaseOutcome {
    pub success: bool,
    pub water_written: bool,
    pub mob_creation_attempted: bool,
    pub mob_admitted: bool,
    pub entity_place_event: bool,
    pub placed_block_criterion: bool,
    pub item_used_stat: bool,
    pub returns_empty_bucket: bool,
    pub retains_filled_bucket: bool,
}

pub const fn release_mob(kind: MobBucketKind, input: ReleaseInput) -> ReleaseOutcome {
    let dry = !kind.contains_water();
    let fluid_success = dry || input.fluid_admitted || input.evaporating_dimension;
    if !input.permissions_admitted || !fluid_success {
        return failed_release();
    }
    let player_use = !input.dispenser;
    ReleaseOutcome {
        success: true,
        water_written: kind.contains_water()
            && input.fluid_admitted
            && !input.evaporating_dimension,
        mob_creation_attempted: input.server_side,
        mob_admitted: input.server_side && input.factory_created && input.admission_accepted,
        entity_place_event: input.server_side,
        placed_block_criterion: input.server_side && player_use && kind.contains_water(),
        item_used_stat: player_use,
        returns_empty_bucket: input.dispenser || !input.infinite_materials,
        retains_filled_bucket: player_use && input.infinite_materials,
    }
}

const fn failed_release() -> ReleaseOutcome {
    ReleaseOutcome {
        success: false,
        water_written: false,
        mob_creation_attempted: false,
        mob_admitted: false,
        entity_place_event: false,
        placed_block_criterion: false,
        item_used_stat: false,
        returns_empty_bucket: false,
        retains_filled_bucket: false,
    }
}
