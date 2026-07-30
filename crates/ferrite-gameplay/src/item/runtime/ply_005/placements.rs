//! Armor-stand, End-crystal, painting, and item-frame placement/runtime rules.

pub const ARMOR_STAND_ENTITY_ID: u32 = 5;
pub const ARMOR_STAND_ITEM_ID: u32 = 1_284;
pub const END_CRYSTAL_ENTITY_ID: u32 = 45;
pub const END_CRYSTAL_ITEM_ID: u32 = 1_312;
pub const PAINTING_ENTITY_ID: u32 = 93;
pub const ITEM_FRAME_ENTITY_ID: u32 = 73;
pub const GLOW_ITEM_FRAME_ENTITY_ID: u32 = 60;
pub const PLACEABLE_PAINTING_VARIANTS: usize = 47;
pub const REGISTERED_PAINTING_VARIANTS: usize = 51;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseResult {
    Fail,
    Consume,
    Success,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Face {
    pub const fn horizontal(self) -> bool {
        matches!(self, Self::North | Self::South | Self::West | Self::East)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArmorStandPlacement {
    pub result: UseResult,
    pub created: bool,
    pub consumed: u8,
    pub yaw: Option<f32>,
    pub configuration_before_final_yaw: bool,
    pub spawn_attempted: bool,
    pub placement_sound: bool,
    pub placement_event: bool,
}

pub fn place_armor_stand(
    face: Face,
    no_collision: bool,
    entity_box_empty: bool,
    server_side: bool,
    creation_succeeded: bool,
    context_rotation: f32,
) -> ArmorStandPlacement {
    if face == Face::Down || !no_collision || !entity_box_empty {
        return armor_stand_failure();
    }
    if server_side && !creation_succeeded {
        return armor_stand_failure();
    }
    ArmorStandPlacement {
        result: UseResult::Success,
        created: server_side,
        consumed: 1,
        yaw: server_side.then(|| quantized_stand_yaw(context_rotation)),
        configuration_before_final_yaw: server_side,
        spawn_attempted: server_side,
        placement_sound: server_side,
        placement_event: server_side,
    }
}

pub fn quantized_stand_yaw(context_rotation: f32) -> f32 {
    let wrapped = wrap_degrees(context_rotation - 180.0);
    ((wrapped + 22.5) / 45.0).floor() * 45.0
}

fn armor_stand_failure() -> ArmorStandPlacement {
    ArmorStandPlacement {
        result: UseResult::Fail,
        created: false,
        consumed: 0,
        yaw: None,
        configuration_before_final_yaw: false,
        spawn_attempted: false,
        placement_sound: false,
        placement_event: false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlot {
    MainHand = 0,
    Feet = 1,
    Legs = 2,
    Chest = 3,
    Head = 4,
    OffHand = 5,
    Body = 6,
    Saddle = 7,
}

pub const fn stand_slot_usable(
    slot: EquipmentSlot,
    occupied: bool,
    show_arms: bool,
    disabled_slots: u32,
) -> bool {
    let id = slot as u32;
    if matches!(slot, EquipmentSlot::Body | EquipmentSlot::Saddle)
        || (matches!(slot, EquipmentSlot::MainHand | EquipmentSlot::OffHand) && !show_arms)
        || disabled_slots & (1 << id) != 0
    {
        return false;
    }
    let transfer_bit = if occupied { id + 8 } else { id + 16 };
    disabled_slots & (1 << transfer_bit) == 0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandOccupancy {
    pub mainhand: bool,
    pub offhand: bool,
    pub feet: bool,
    pub legs: bool,
    pub chest: bool,
    pub head: bool,
}

pub const fn select_stand_slot_for_empty_hand(
    hit_y_after_scale: f64,
    small: bool,
    occupied: StandOccupancy,
    disabled_slots: u32,
) -> EquipmentSlot {
    let selected = if hit_y_after_scale >= 0.1
        && hit_y_after_scale < if small { 0.9 } else { 0.55 }
        && occupied.feet
    {
        EquipmentSlot::Feet
    } else if hit_y_after_scale >= 0.9
        && hit_y_after_scale < if small { 1.9 } else { 1.6 }
        && occupied.chest
    {
        EquipmentSlot::Chest
    } else if hit_y_after_scale >= 0.4
        && hit_y_after_scale < if small { 1.4 } else { 1.2 }
        && occupied.legs
    {
        EquipmentSlot::Legs
    } else if hit_y_after_scale >= 1.6 && occupied.head {
        EquipmentSlot::Head
    } else if occupied.mainhand {
        EquipmentSlot::MainHand
    } else if occupied.offhand {
        EquipmentSlot::OffHand
    } else {
        EquipmentSlot::MainHand
    };
    if disabled_slots & (1 << selected as u32) == 0 {
        selected
    } else {
        EquipmentSlot::MainHand
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandSwapOutcome {
    pub success: bool,
    pub hand_count: u32,
    pub slot_count: u32,
    pub copied_for_infinite_materials: bool,
}

pub const fn swap_stand_slot(
    hand_count: u32,
    slot_count: u32,
    infinite_materials: bool,
    slot_usable: bool,
) -> StandSwapOutcome {
    if !slot_usable || (hand_count == 0 && slot_count == 0) {
        return StandSwapOutcome {
            success: false,
            hand_count,
            slot_count,
            copied_for_infinite_materials: false,
        };
    }
    if infinite_materials && hand_count > 0 && slot_count == 0 {
        return StandSwapOutcome {
            success: true,
            hand_count,
            slot_count: 1,
            copied_for_infinite_materials: true,
        };
    }
    if hand_count > 1 {
        if slot_count != 0 {
            return StandSwapOutcome {
                success: false,
                hand_count,
                slot_count,
                copied_for_infinite_materials: false,
            };
        }
        return StandSwapOutcome {
            success: true,
            hand_count: hand_count - 1,
            slot_count: 1,
            copied_for_infinite_materials: false,
        };
    }
    StandSwapOutcome {
        success: true,
        hand_count: slot_count,
        slot_count: hand_count,
        copied_for_infinite_materials: false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandDamageKind {
    Bypass,
    Explosion,
    Ignite,
    Burn,
    Break,
    AlwaysKill,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandDamageOutcome {
    Rejected,
    RemovedWithoutDrops,
    ExplosionBreak,
    Ignited,
    FireDamaged,
    FirstHit,
    BrokenWithStandItem,
    CreativeBreak,
}

pub const fn damage_armor_stand(
    kind: StandDamageKind,
    game_time: u64,
    last_hit: u64,
    burning: bool,
    health_above_half: bool,
    creative_player: bool,
    may_build: bool,
) -> StandDamageOutcome {
    match kind {
        StandDamageKind::Bypass => StandDamageOutcome::RemovedWithoutDrops,
        StandDamageKind::Explosion => StandDamageOutcome::ExplosionBreak,
        StandDamageKind::Ignite if !burning => StandDamageOutcome::Ignited,
        StandDamageKind::Ignite | StandDamageKind::Burn if health_above_half => {
            StandDamageOutcome::FireDamaged
        }
        StandDamageKind::Ignite | StandDamageKind::Burn => StandDamageOutcome::ExplosionBreak,
        StandDamageKind::Break | StandDamageKind::AlwaysKill if !may_build => {
            StandDamageOutcome::Rejected
        }
        StandDamageKind::Break | StandDamageKind::AlwaysKill if creative_player => {
            StandDamageOutcome::CreativeBreak
        }
        StandDamageKind::AlwaysKill => StandDamageOutcome::BrokenWithStandItem,
        StandDamageKind::Break if game_time.saturating_sub(last_hit) <= 5 => {
            StandDamageOutcome::BrokenWithStandItem
        }
        StandDamageKind::Break => StandDamageOutcome::FirstHit,
        StandDamageKind::Other => StandDamageOutcome::Rejected,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrystalBase {
    Obsidian,
    Bedrock,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndCrystalPlacement {
    pub result: UseResult,
    pub consumed: u8,
    pub created: bool,
    pub show_bottom: bool,
    pub placement_event: bool,
    pub respawn_check: bool,
}

pub const fn place_end_crystal(
    base: CrystalBase,
    lower_cell_empty: bool,
    two_cell_box_empty: bool,
    server_side: bool,
) -> EndCrystalPlacement {
    if matches!(base, CrystalBase::Other) || !lower_cell_empty || !two_cell_box_empty {
        return EndCrystalPlacement {
            result: UseResult::Fail,
            consumed: 0,
            created: false,
            show_bottom: true,
            placement_event: false,
            respawn_check: false,
        };
    }
    EndCrystalPlacement {
        result: UseResult::Success,
        consumed: 1,
        created: server_side,
        show_bottom: false,
        placement_event: server_side,
        respawn_check: server_side,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrystalDamageOutcome {
    pub accepted: bool,
    pub removed: bool,
    pub explosion_radius: Option<u8>,
    pub fight_notified_after_explosion: bool,
}

pub fn damage_end_crystal(
    server_side: bool,
    invulnerable: bool,
    responsible_is_dragon: bool,
    already_removed: bool,
    incoming_explosion: bool,
) -> CrystalDamageOutcome {
    if invulnerable || responsible_is_dragon {
        return CrystalDamageOutcome {
            accepted: false,
            removed: false,
            explosion_radius: None,
            fight_notified_after_explosion: false,
        };
    }
    CrystalDamageOutcome {
        accepted: true,
        removed: server_side && !already_removed,
        explosion_radius: (server_side && !already_removed && !incoming_explosion).then_some(6),
        fight_notified_after_explosion: server_side && !already_removed,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HangingKind {
    Painting,
    ItemFrame,
    GlowItemFrame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaintingCandidate {
    pub area: u16,
    pub survives: bool,
}

pub fn maximal_painting_candidate(
    candidates: &[PaintingCandidate],
    random_index: usize,
) -> Option<usize> {
    let maximum = candidates
        .iter()
        .filter(|candidate| candidate.survives)
        .map(|candidate| candidate.area)
        .max()?;
    let maximal = candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| candidate.survives && candidate.area == maximum)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    maximal.get(random_index % maximal.len()).copied()
}

pub const fn hanging_admitted(kind: HangingKind, player_present: bool, face: Face) -> bool {
    if !player_present {
        true
    } else {
        face.horizontal() && (!matches!(kind, HangingKind::Painting) || face.horizontal())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HangingPlacement {
    pub result: UseResult,
    pub consumed: u8,
    pub spawn_attempted: bool,
    pub sound_and_event: bool,
}

pub const fn finalize_hanging_placement(
    candidate_exists: bool,
    final_survival: bool,
    server_side: bool,
) -> HangingPlacement {
    if !candidate_exists || !final_survival {
        return HangingPlacement {
            result: UseResult::Consume,
            consumed: 0,
            spawn_attempted: false,
            sound_and_event: false,
        };
    }
    HangingPlacement {
        result: UseResult::Success,
        consumed: 1,
        spawn_attempted: server_side,
        sound_and_event: server_side,
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SurvivalCadence {
    counter: u32,
}

impl SurvivalCadence {
    pub fn tick(&mut self) -> bool {
        let check = self.counter == 100;
        self.counter = if check {
            0
        } else {
            self.counter.saturating_add(1)
        };
        check
    }
}

pub const fn frame_map_admitted(tracked_decorations: Option<usize>) -> bool {
    match tracked_decorations {
        Some(count) => count <= 256,
        None => true,
    }
}

pub const fn rotate_frame(rotation: i32) -> i32 {
    (rotation + 1) % 8
}

pub const fn frame_analog_value(empty: bool, rotation: i32) -> i32 {
    if empty { 0 } else { rotation % 8 + 1 }
}

fn wrap_degrees(value: f32) -> f32 {
    let mut wrapped = value % 360.0;
    if wrapped >= 180.0 {
        wrapped -= 360.0;
    }
    if wrapped < -180.0 {
        wrapped += 360.0;
    }
    wrapped
}
