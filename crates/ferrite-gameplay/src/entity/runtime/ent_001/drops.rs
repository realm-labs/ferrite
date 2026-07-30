//! Ordered `entity_drops` decisions at the seven live read sites.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropEffect {
    CarrierKilled,
    CarrierItemSpawned,
    ContainerSlotVisited(usize),
    ContainerContentsDropped,
    PiglinsAngered,
    PaintingBreakSound,
    PaintingItemSpawned,
    FrameItemCleared,
    FrameMapRemoved,
    FrameItemSpawned,
    DisplayedItemSpawned,
    LeashDataCleared,
    LeashRemovedCallback,
    LeashLinkPacket,
    LeashHolderNotified,
    LeadSpawned,
    FallingEntityDiscarded,
    FallingBrokenHook,
    FallingBlockItemSpawned,
    StatueCommitted,
    PreservedEquipmentDropped,
}

#[must_use]
pub fn destroy_vehicle(entity_drops: bool) -> Vec<DropEffect> {
    let mut effects = vec![DropEffect::CarrierKilled];
    if entity_drops {
        effects.push(DropEffect::CarrierItemSpawned);
    }
    effects
}

#[must_use]
pub fn destroy_container_vehicle(
    entity_drops: bool,
    occupied_slots: &[bool],
    direct_entity_is_player: bool,
) -> Vec<DropEffect> {
    if !entity_drops {
        return Vec::new();
    }
    let mut effects = Vec::with_capacity(occupied_slots.len() * 2 + 1);
    for (index, occupied) in occupied_slots.iter().copied().enumerate() {
        effects.push(DropEffect::ContainerSlotVisited(index));
        if occupied {
            effects.push(DropEffect::ContainerContentsDropped);
        }
    }
    if direct_entity_is_player {
        effects.push(DropEffect::PiglinsAngered);
    }
    effects
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerSplit {
    pub piece_counts: Vec<u16>,
    pub position_double_draws: u8,
    pub velocity_double_draws: usize,
}

pub fn split_container_slot(stack_count: u16, split_draws: &[u8]) -> Option<ContainerSplit> {
    let mut remaining = stack_count;
    let mut pieces = Vec::new();
    let mut draws = split_draws.iter().copied();
    while remaining > 0 {
        let draw = draws.next()?;
        let requested = 10 + u16::from(draw % 21);
        let piece = requested.min(remaining);
        pieces.push(piece);
        remaining -= piece;
    }
    Some(ContainerSplit {
        velocity_double_draws: pieces.len() * 6,
        piece_counts: pieces,
        position_double_draws: 3,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Remover {
    None,
    Ordinary,
    InfiniteMaterials,
}

#[must_use]
pub fn drop_painting(entity_drops: bool, remover: Remover) -> Vec<DropEffect> {
    if !entity_drops {
        return Vec::new();
    }
    let mut effects = vec![DropEffect::PaintingBreakSound];
    if !matches!(remover, Remover::InfiniteMaterials) {
        effects.push(DropEffect::PaintingItemSpawned);
    }
    effects
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameDropInput {
    pub fixed: bool,
    pub entity_drops: bool,
    pub remover: Remover,
    pub drop_frame: bool,
    pub displayed_item: bool,
    pub displayed_map: bool,
    pub drop_chance: f32,
    pub draw: f32,
}

#[must_use]
pub fn drop_frame(input: FrameDropInput) -> Vec<DropEffect> {
    if input.fixed {
        return Vec::new();
    }
    let mut effects = vec![DropEffect::FrameItemCleared];
    if !input.entity_drops {
        if matches!(input.remover, Remover::None) && input.displayed_map {
            effects.push(DropEffect::FrameMapRemoved);
        }
        return effects;
    }
    if matches!(input.remover, Remover::InfiniteMaterials) {
        if input.displayed_map {
            effects.push(DropEffect::FrameMapRemoved);
        }
        return effects;
    }
    if input.drop_frame {
        effects.push(DropEffect::FrameItemSpawned);
    }
    if input.displayed_item {
        if input.displayed_map {
            effects.push(DropEffect::FrameMapRemoved);
        }
        if input.draw < input.drop_chance {
            effects.push(DropEffect::DisplayedItemSpawned);
        }
    }
    effects
}

#[must_use]
pub fn detach_invalid_leash(entity_drops: bool) -> Vec<DropEffect> {
    let mut effects = vec![
        DropEffect::LeashDataCleared,
        DropEffect::LeashRemovedCallback,
        DropEffect::LeashLinkPacket,
        DropEffect::LeashHolderNotified,
    ];
    if entity_drops {
        effects.push(DropEffect::LeadSpawned);
    }
    effects
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallingFailure {
    Cancelled,
    PlacementWriteFailed,
    Ineligible,
    TimedOut,
}

#[must_use]
pub fn falling_failure(
    failure: FallingFailure,
    drop_item: bool,
    entity_drops: bool,
) -> Vec<DropEffect> {
    if matches!(failure, FallingFailure::Cancelled) {
        return vec![
            DropEffect::FallingEntityDiscarded,
            DropEffect::FallingBrokenHook,
        ];
    }
    let drops = drop_item && entity_drops;
    match failure {
        FallingFailure::PlacementWriteFailed if !drops => Vec::new(),
        FallingFailure::PlacementWriteFailed => vec![
            DropEffect::FallingEntityDiscarded,
            DropEffect::FallingBrokenHook,
            DropEffect::FallingBlockItemSpawned,
        ],
        FallingFailure::Ineligible if !drops => vec![DropEffect::FallingEntityDiscarded],
        FallingFailure::Ineligible => vec![
            DropEffect::FallingEntityDiscarded,
            DropEffect::FallingBrokenHook,
            DropEffect::FallingBlockItemSpawned,
        ],
        FallingFailure::TimedOut if !drops => vec![DropEffect::FallingEntityDiscarded],
        FallingFailure::TimedOut => vec![
            DropEffect::FallingEntityDiscarded,
            DropEffect::FallingBlockItemSpawned,
        ],
        FallingFailure::Cancelled => unreachable!("cancelled returned before drop gates"),
    }
}

#[must_use]
pub fn convert_leashed_statue(entity_drops: bool) -> Vec<DropEffect> {
    let mut effects = vec![
        DropEffect::StatueCommitted,
        DropEffect::PreservedEquipmentDropped,
    ];
    effects.extend(detach_invalid_leash(entity_drops));
    effects
}
