//! Falling-block state transfer, landing, timeout, and subtype decisions.

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::id::BlockStateId;

pub const GENERIC_FALL_DELAY: u32 = 2;
pub const DRAGON_EGG_FALL_DELAY: u32 = 5;
pub const SCAFFOLDING_FALL_DELAY: u32 = 1;
pub const FALLING_WRITE_FLAGS: u16 = 3;
pub const DRAGON_EGG_TELEPORT_FLAGS: u16 = 2;
pub const GRAVITY: f64 = 0.04;
pub const DRAG: f64 = 0.98;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnvilStage {
    Intact,
    Chipped,
    Damaged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallingKind {
    Generic,
    ConcretePowder,
    Anvil(AnvilStage),
    DragonEgg,
    Brushable,
    Scaffolding,
}

pub fn falling_kind(path: &str) -> Option<FallingKind> {
    let kind = match path {
        "sand" | "red_sand" | "gravel" => FallingKind::Generic,
        "white_concrete_powder"
        | "orange_concrete_powder"
        | "magenta_concrete_powder"
        | "light_blue_concrete_powder"
        | "yellow_concrete_powder"
        | "lime_concrete_powder"
        | "pink_concrete_powder"
        | "gray_concrete_powder"
        | "light_gray_concrete_powder"
        | "cyan_concrete_powder"
        | "purple_concrete_powder"
        | "blue_concrete_powder"
        | "brown_concrete_powder"
        | "green_concrete_powder"
        | "red_concrete_powder"
        | "black_concrete_powder" => FallingKind::ConcretePowder,
        "anvil" => FallingKind::Anvil(AnvilStage::Intact),
        "chipped_anvil" => FallingKind::Anvil(AnvilStage::Chipped),
        "damaged_anvil" => FallingKind::Anvil(AnvilStage::Damaged),
        "dragon_egg" => FallingKind::DragonEgg,
        "suspicious_sand" | "suspicious_gravel" => FallingKind::Brushable,
        "scaffolding" => FallingKind::Scaffolding,
        _ => return None,
    };
    Some(kind)
}

pub const fn fall_delay(kind: FallingKind) -> u32 {
    match kind {
        FallingKind::DragonEgg => DRAGON_EGG_FALL_DELAY,
        FallingKind::Scaffolding => SCAFFOLDING_FALL_DELAY,
        FallingKind::Generic
        | FallingKind::ConcretePowder
        | FallingKind::Anvil(_)
        | FallingKind::Brushable => GENERIC_FALL_DELAY,
    }
}

pub const fn should_start_fall(origin_y: i32, minimum_y: i32, below_is_free: bool) -> bool {
    below_is_free && origin_y >= minimum_y
}

pub const fn falling_entity_position(origin: BlockPos) -> [f64; 3] {
    [
        origin.x as f64 + 0.5,
        origin.y as f64,
        origin.z as f64 + 0.5,
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallStartEffect {
    ResetBrushableBlockEntity,
    CreateEntityAtBlockCenter,
    ClearCarriedWaterlogged,
    RecordStartPosition,
    ReplaceOriginWithFluid { flags: u16 },
    SetCancelDrop,
    OfferEntityAdmission,
}

pub fn plan_fall_start(kind: FallingKind) -> Vec<FallStartEffect> {
    let mut effects = Vec::new();
    if kind == FallingKind::Brushable {
        effects.push(FallStartEffect::ResetBrushableBlockEntity);
    }
    effects.extend([
        FallStartEffect::CreateEntityAtBlockCenter,
        FallStartEffect::ClearCarriedWaterlogged,
        FallStartEffect::RecordStartPosition,
        FallStartEffect::ReplaceOriginWithFluid {
            flags: FALLING_WRITE_FLAGS,
        },
    ]);
    effects.push(FallStartEffect::OfferEntityAdmission);
    if kind == FallingKind::Brushable {
        effects.push(FallStartEffect::SetCancelDrop);
    }
    effects
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FallingEntity {
    pub carried_state: BlockStateId,
    pub carried_is_air: bool,
    pub time: i32,
    pub position_y: f64,
    pub velocity: [f64; 3],
    pub drop_item: bool,
    pub cancel_drop: bool,
    pub hurts_entities: bool,
    pub fall_damage_amount: f32,
    pub fall_damage_maximum: i32,
    pub carried_block_entity_data: bool,
}

impl FallingEntity {
    pub const fn new(carried_state: BlockStateId, carried_is_air: bool, position_y: f64) -> Self {
        Self {
            carried_state,
            carried_is_air,
            time: 0,
            position_y,
            velocity: [0.0; 3],
            drop_item: true,
            cancel_drop: false,
            hurts_entities: false,
            fall_damage_amount: 0.0,
            fall_damage_maximum: 40,
            carried_block_entity_data: false,
        }
    }

    pub const fn configure_anvil_damage(&mut self) {
        self.hurts_entities = true;
        self.fall_damage_amount = 2.0;
        self.fall_damage_maximum = 40;
    }

    pub fn begin_tick(&mut self) -> Vec<FallingTickEffect> {
        if self.carried_is_air {
            return vec![FallingTickEffect::DiscardAirState];
        }
        self.time = self.time.wrapping_add(1);
        self.velocity[1] -= GRAVITY;
        vec![
            FallingTickEffect::IncrementTime,
            FallingTickEffect::ApplyGravity,
            FallingTickEffect::MoveSelf,
            FallingTickEffect::ApplyBlockEffectsAndPortals,
        ]
    }

    pub fn finish_tick(&mut self) -> FallingTickEffect {
        for velocity in &mut self.velocity {
            *velocity *= DRAG;
        }
        FallingTickEffect::ApplyDrag
    }

    pub const fn timed_out(self, minimum_y: i32, maximum_y: i32) -> bool {
        (self.time > 100
            && (self.position_y <= minimum_y as f64 || self.position_y > maximum_y as f64))
            || self.time > 600
    }

    pub fn timeout_effects(self, do_entity_drops: bool) -> Vec<FallingTickEffect> {
        let mut effects = Vec::new();
        if self.drop_item && do_entity_drops {
            effects.push(FallingTickEffect::SpawnCarriedItem);
        }
        effects.push(FallingTickEffect::DiscardTimeout);
        effects
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallingTickEffect {
    DiscardAirState,
    IncrementTime,
    ApplyGravity,
    MoveSelf,
    ApplyBlockEffectsAndPortals,
    ApplyLandingVelocity,
    DeferAboveMovingPiston,
    BrokenHookWithoutItem,
    CopyDestinationWaterlogged,
    AttemptPlacement { flags: u16 },
    SendTrackingBlockUpdate,
    OverlaySerializedBlockEntityData,
    OnLand,
    BrokenHookWithItem,
    SpawnCarriedItem,
    DiscardLanding,
    DiscardTimeout,
    ApplyDrag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandingInputs {
    pub moving_piston_target: bool,
    pub cancel_drop: bool,
    pub target_replaceable: bool,
    pub carried_state_survives: bool,
    pub block_below_still_free: bool,
    pub water_contact_bypasses_below_free: bool,
    pub placement_succeeded: bool,
    pub drop_item: bool,
    pub do_entity_drops: bool,
    pub serialized_block_entity_data: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LandingPlan {
    pub remains_active: bool,
    pub effects: Vec<FallingTickEffect>,
}

pub fn plan_landing(inputs: LandingInputs) -> LandingPlan {
    let mut effects = vec![FallingTickEffect::ApplyLandingVelocity];
    if inputs.moving_piston_target {
        effects.push(FallingTickEffect::DeferAboveMovingPiston);
        return LandingPlan {
            remains_active: true,
            effects,
        };
    }
    if inputs.cancel_drop {
        effects.extend([
            FallingTickEffect::DiscardLanding,
            FallingTickEffect::BrokenHookWithoutItem,
        ]);
        return LandingPlan {
            remains_active: false,
            effects,
        };
    }

    let eligible = inputs.target_replaceable
        && inputs.carried_state_survives
        && (!inputs.block_below_still_free || inputs.water_contact_bypasses_below_free);
    if eligible {
        effects.extend([
            FallingTickEffect::CopyDestinationWaterlogged,
            FallingTickEffect::AttemptPlacement {
                flags: FALLING_WRITE_FLAGS,
            },
        ]);
        if inputs.placement_succeeded {
            effects.extend([
                FallingTickEffect::SendTrackingBlockUpdate,
                FallingTickEffect::DiscardLanding,
                FallingTickEffect::OnLand,
            ]);
            if inputs.serialized_block_entity_data {
                effects.push(FallingTickEffect::OverlaySerializedBlockEntityData);
            }
            return LandingPlan {
                remains_active: false,
                effects,
            };
        }
        if !inputs.drop_item || !inputs.do_entity_drops {
            return LandingPlan {
                remains_active: true,
                effects,
            };
        }
    }

    effects.push(FallingTickEffect::DiscardLanding);
    if inputs.drop_item && inputs.do_entity_drops {
        effects.extend([
            FallingTickEffect::BrokenHookWithItem,
            FallingTickEffect::SpawnCarriedItem,
        ]);
    }
    LandingPlan {
        remains_active: false,
        effects,
    }
}

pub const fn landing_velocity(velocity: [f64; 3]) -> [f64; 3] {
    [velocity[0] * 0.7, velocity[1] * -0.5, velocity[2] * 0.7]
}

pub const fn should_finish_removed_tick(removed: bool, teleported_through_end: bool) -> bool {
    !removed || teleported_through_end
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallingHurtEffect {
    RecordLastHurt,
}

pub const fn falling_entity_hurt() -> (bool, FallingHurtEffect) {
    (false, FallingHurtEffect::RecordLastHurt)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FallingEntitySnapshot {
    pub persistent_id: u128,
    pub entity: FallingEntity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallingUnloadEffect {
    StoreEntity,
    RemoveTickCallback,
}

pub const FALLING_UNLOAD_EFFECTS: [FallingUnloadEffect; 2] = [
    FallingUnloadEffect::StoreEntity,
    FallingUnloadEffect::RemoveTickCallback,
];

pub const fn restore_snapshot(
    snapshot: FallingEntitySnapshot,
    persistent_id_available: bool,
) -> Option<FallingEntity> {
    if persistent_id_available {
        Some(snapshot.entity)
    } else {
        None
    }
}

pub const fn loaded_hurts_entities(carried_is_anvil: bool, saved_value: Option<bool>) -> bool {
    match saved_value {
        Some(value) => value,
        None => carried_is_anvil,
    }
}

pub const fn anvil_target_is_hurt(
    alive: bool,
    creative: bool,
    spectator: bool,
    intersects_entity_box: bool,
) -> bool {
    alive && !creative && !spectator && intersects_entity_box
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnvilImpact {
    pub distance_index: i32,
    pub damage: i32,
    pub degradation_threshold: Option<f32>,
}

pub fn anvil_impact(fall_distance: f32, damage_amount: f32, maximum_damage: i32) -> AnvilImpact {
    let distance_index = (fall_distance - 1.0).ceil() as i32;
    if distance_index < 0 {
        return AnvilImpact {
            distance_index,
            damage: 0,
            degradation_threshold: None,
        };
    }
    let damage = ((damage_amount * distance_index as f32).floor() as i32).min(maximum_damage);
    AnvilImpact {
        distance_index,
        damage,
        degradation_threshold: (damage > 0).then_some(0.05 + 0.05 * distance_index as f32),
    }
}

pub const fn degrade_anvil(
    stage: AnvilStage,
    degradation_draw: f32,
    threshold: f32,
) -> (AnvilStage, bool) {
    if degradation_draw >= threshold {
        return (stage, false);
    }
    match stage {
        AnvilStage::Intact => (AnvilStage::Chipped, false),
        AnvilStage::Chipped => (AnvilStage::Damaged, false),
        AnvilStage::Damaged => (AnvilStage::Damaged, true),
    }
}

pub const fn concrete_powder_uses_water_hit(
    velocity_length_squared: f64,
    source_fluid_raycast_hit: bool,
) -> bool {
    velocity_length_squared > 1.0 && source_fluid_raycast_hit
}

pub const fn concrete_powder_solidifies(
    water_at_position: bool,
    neighboring_water_with_nonsturdy_face: bool,
) -> bool {
    water_at_position || neighboring_water_with_nonsturdy_face
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubtypeFallEffect {
    ConfigureAnvilDamage { amount: u8, maximum: u8 },
    SolidifyConcrete,
    AnvilLandEvent(i32),
    AnvilBrokenEvent(i32),
    BrushableDestroyEvent(i32),
    BlockDestroyGameEvent,
}

pub const fn start_subtype_effect(kind: FallingKind) -> Option<SubtypeFallEffect> {
    if matches!(kind, FallingKind::Anvil(_)) {
        Some(SubtypeFallEffect::ConfigureAnvilDamage {
            amount: 2,
            maximum: 40,
        })
    } else {
        None
    }
}

pub fn on_land_subtype_effects(
    kind: FallingKind,
    prelanding_concrete_solidifies: bool,
    silent: bool,
) -> Vec<SubtypeFallEffect> {
    match kind {
        FallingKind::ConcretePowder if prelanding_concrete_solidifies => {
            vec![SubtypeFallEffect::SolidifyConcrete]
        }
        FallingKind::Anvil(_) if !silent => {
            vec![SubtypeFallEffect::AnvilLandEvent(1031)]
        }
        FallingKind::Generic
        | FallingKind::ConcretePowder
        | FallingKind::Anvil(_)
        | FallingKind::DragonEgg
        | FallingKind::Brushable
        | FallingKind::Scaffolding => Vec::new(),
    }
}

pub fn on_broken_subtype_effects(kind: FallingKind, silent: bool) -> Vec<SubtypeFallEffect> {
    match kind {
        FallingKind::Anvil(_) if !silent => {
            vec![SubtypeFallEffect::AnvilBrokenEvent(1029)]
        }
        FallingKind::Brushable => vec![
            SubtypeFallEffect::BrushableDestroyEvent(2001),
            SubtypeFallEffect::BlockDestroyGameEvent,
        ],
        FallingKind::Generic
        | FallingKind::ConcretePowder
        | FallingKind::Anvil(_)
        | FallingKind::DragonEgg
        | FallingKind::Scaffolding => Vec::new(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScaffoldingDistance {
    pub distance: u8,
    pub bottom: bool,
}

pub fn scaffolding_distance(
    sturdy_below: bool,
    below_scaffolding_distance: Option<u8>,
    horizontal_scaffolding_distances: &[u8],
) -> ScaffoldingDistance {
    let distance = if sturdy_below {
        0
    } else if let Some(below) = below_scaffolding_distance {
        below
    } else {
        horizontal_scaffolding_distances
            .iter()
            .copied()
            .min()
            .map_or(7, |distance| distance.saturating_add(1).min(7))
    };
    ScaffoldingDistance {
        distance,
        bottom: distance > 0 && below_scaffolding_distance.is_none(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaffoldingTick {
    DestroyWithDrops,
    SpawnFallingEntity,
    WriteSupportedState { flags: u16 },
    NoChange,
}

pub const fn scaffolding_tick(
    previous_distance: u8,
    previous_bottom: bool,
    recomputed: ScaffoldingDistance,
) -> ScaffoldingTick {
    if recomputed.distance == 7 {
        if previous_distance == 7 {
            ScaffoldingTick::SpawnFallingEntity
        } else {
            ScaffoldingTick::DestroyWithDrops
        }
    } else if previous_distance != recomputed.distance || previous_bottom != recomputed.bottom {
        ScaffoldingTick::WriteSupportedState {
            flags: FALLING_WRITE_FLAGS,
        }
    } else {
        ScaffoldingTick::NoChange
    }
}

pub const fn scaffolding_survives(distance: u8) -> bool {
    distance < 7
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DragonEggCandidate {
    pub candidate_is_air: bool,
    pub below_is_nonair: bool,
    pub inside_build_height: bool,
    pub inside_world_border: bool,
}

impl DragonEggCandidate {
    pub const fn valid(self) -> bool {
        self.candidate_is_air
            && self.below_is_nonair
            && self.inside_build_height
            && self.inside_world_border
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragonEggEffect {
    WriteCandidate { flags: u16 },
    RemoveOriginWithoutDrops,
    EmitPortalParticles(u16),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragonEggTeleport {
    pub attempts: usize,
    pub random_draws: usize,
    pub accepted_candidate: Option<usize>,
    pub effects: Vec<DragonEggEffect>,
}

pub fn plan_dragon_egg_teleport(
    client_side: bool,
    candidates: &[DragonEggCandidate],
) -> DragonEggTeleport {
    let examined = candidates.iter().take(1_000);
    let accepted = examined.clone().position(|candidate| candidate.valid());
    let attempts = accepted.map_or_else(
        || candidates.len().min(1_000),
        |accepted_index| accepted_index + 1,
    );
    let effects = if accepted.is_some() {
        if client_side {
            vec![DragonEggEffect::EmitPortalParticles(128)]
        } else {
            vec![
                DragonEggEffect::WriteCandidate {
                    flags: DRAGON_EGG_TELEPORT_FLAGS,
                },
                DragonEggEffect::RemoveOriginWithoutDrops,
            ]
        }
    } else {
        Vec::new()
    };
    DragonEggTeleport {
        attempts,
        random_draws: attempts * 6,
        accepted_candidate: accepted,
        effects,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AmbientDust {
    pub random_draws: usize,
    pub emitted: bool,
}

pub const fn ambient_dust(next_int_16: u8) -> AmbientDust {
    AmbientDust {
        random_draws: 1,
        emitted: next_int_16 == 0,
    }
}
